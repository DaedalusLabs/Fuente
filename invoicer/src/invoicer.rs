use anyhow::anyhow;
use bright_lightning::{
    HodlState, InvoicePaymentState, LnAddressPaymentRequest, LndHodlInvoice, LndHodlInvoiceState,
    LndPaymentRequest, LndPaymentResponse, LndWebsocketMessage,
};
use fuente::models::{
    CommerceProfile, OrderInvoiceState, OrderParticipant, OrderPaymentStatus, OrderRequest,
    OrderStatus,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote, relays::PoolRelayBroadcaster};
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

use crate::{state::InvoicerStateLock, InvoicerBot};
pub const SATOSHIS_IN_ONE_BTC: f64 = 100_000_000.0;
pub const MILISATOSHIS_IN_ONE_SATOSHI: u64 = 1000;
pub const ILLUMINODES_FEES: u64 = 20;
pub const FUENTE_FEES: u64 = 0;

#[derive(Clone)]
pub struct Invoicer {
    rest_client: reqwest::Client,
    lightning_wallet: bright_lightning::LightningClient,
}
impl Invoicer {
    pub async fn new() -> anyhow::Result<Self> {
        let rest_client = reqwest::Client::new();
        let lightning_wallet = bright_lightning::LightningClient::new(
            Box::leak(
                std::env::var("LND_ADDRESS")
                    .expect("LND_ADDRESS not set")
                    .into_boxed_str(),
            ),
            Box::leak(
                std::env::var("LND_MACAROON")
                    .expect("LND_MACAROON not set")
                    .into_boxed_str(),
            ),
        )
        .await?;
        Ok(Self {
            rest_client,
            lightning_wallet,
        })
    }
    pub async fn create_order_invoice(
        &self,
        order: &OrderRequest,
        commerce_profile: &CommerceProfile,
        exchange_rate: f64,
    ) -> anyhow::Result<(LnAddressPaymentRequest, LndHodlInvoice)> {
        // TODO
        // verify products agains signed prodcut list

        let invoice_total_srd = order.products.total();
        let invoice_satoshi_amount = invoice_total_srd / exchange_rate * SATOSHIS_IN_ONE_BTC;
        let invoice = match commerce_profile
            .ln_address()
            .get_invoice(
                &self.rest_client,
                invoice_satoshi_amount as u64 * MILISATOSHIS_IN_ONE_SATOSHI,
            )
            .await
        {
            Ok(invoice) => invoice,
            Err(e) => {
                error!("{:?}", e);
                return Err(anyhow!("Could not create invoice"));
            }
        };
        info!("commerce Invoice created");
        let hodl_amount = invoice_satoshi_amount as u64 + ILLUMINODES_FEES + FUENTE_FEES;
        info!("Hodl amount: {}", hodl_amount);
        let hodl_invoice = self
            .lightning_wallet
            .get_hodl_invoice(invoice.r_hash()?, hodl_amount)
            .await?;
        info!("Hodl invoice created");
        Ok((invoice, hodl_invoice))
    }
    pub async fn order_payment_notifier(
        self,
        order_invoice: OrderInvoiceState,
        keys: NostrKeypair,
        state_clone: InvoicerStateLock,
        broadcaster: PoolRelayBroadcaster,
    ) -> anyhow::Result<()> {
        let invoice = order_invoice
            .commerce_invoice
            .as_ref()
            .ok_or(anyhow!("No invoice"))?;
        let mut subscriber = self
            .lightning_wallet
            .subscribe_to_invoice(invoice.r_hash_url_safe()?)
            .await?;
        let mut iter = subscriber.event_stream::<LndHodlInvoiceState>();
        let mut ping_counter = 0;
        while let Some(payment_response) = iter.next().await {
            match payment_response {
                LndWebsocketMessage::Response(invoice_state) => match invoice_state.state() {
                    HodlState::OPEN => {
                        let signed_order =
                            order_invoice.sign_update_for(OrderParticipant::Consumer, &keys)?;
                        broadcaster.broadcast_note(signed_order).await?;
                        state_clone.add_live_order(order_invoice.clone()).await?;
                    }
                    HodlState::ACCEPTED => {
                        if let Some(mut live_order) = state_clone
                            .find_live_order(order_invoice.order_id().as_str())
                            .await
                        {
                            live_order.payment_status = OrderPaymentStatus::PaymentReceived;
                            state_clone.update_live_order(live_order.clone()).await?;
                            InvoicerBot::broadcast_order_update(
                                broadcaster.clone(),
                                keys.clone(),
                                &live_order,
                            )
                            .await?;
                        }
                    }
                    HodlState::SETTLED => {
                        if let Some(mut live_order) = state_clone
                            .find_live_order(order_invoice.order_id().as_str())
                            .await
                        {
                            live_order.payment_status = OrderPaymentStatus::PaymentSuccess;
                            live_order.order_status = OrderStatus::Preparing;
                            state_clone.update_live_order(live_order.clone()).await?;
                            InvoicerBot::broadcast_order_update(
                                broadcaster.clone(),
                                keys.clone(),
                                &live_order,
                            )
                            .await?;
                        }
                    }
                    HodlState::CANCELED => {
                        if let Some(mut live_order) = state_clone
                            .find_live_order(order_invoice.order_id().as_str())
                            .await
                        {
                            live_order.payment_status = OrderPaymentStatus::PaymentFailed;
                            live_order.order_status = OrderStatus::Canceled;
                            state_clone.update_live_order(live_order.clone()).await?;
                            InvoicerBot::broadcast_order_update(
                                broadcaster.clone(),
                                keys.clone(),
                                &live_order,
                            )
                            .await?;
                        }
                    }
                },
                LndWebsocketMessage::Error(e) => {
                    error!("{:?}", e);
                    self.cancel_htlc(invoice.clone()).await?;
                    break;
                }
                _ => {
                    ping_counter += 1;
                    if ping_counter > 5 {
                        warn!("Canceling HTLC due to inactivity");
                        self.cancel_htlc(invoice.clone()).await?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    pub async fn new_order_invoice(
        &self,
        order: OrderRequest,
        signed_note: NostrNote,
        commerce: CommerceProfile,
        exchange_rate: f64,
        keys: NostrKeypair,
        state_clone: InvoicerStateLock,
        broadcaster: PoolRelayBroadcaster,
    ) -> anyhow::Result<OrderInvoiceState> {
        info!("Order request received");
        let invoice = self
            .create_order_invoice(&order, &commerce, exchange_rate)
            .await?;
        info!("Order invoice created");
        let state_update = OrderInvoiceState::new(
            signed_note.clone(),
            Some(invoice.1),
            Some(invoice.0.clone()),
        );
        info!("Order state created");
        let task = self.clone().order_payment_notifier(
            state_update.clone(),
            keys,
            state_clone,
            broadcaster,
        );
        tokio::task::spawn(task);
        Ok(state_update)
    }
    pub async fn cancel_htlc(&self, invoice: LnAddressPaymentRequest) -> anyhow::Result<()> {
        self.lightning_wallet
            .cancel_htlc(invoice.r_hash_url_safe()?)
            .await?;
        Ok(())
    }
    pub async fn settle_htlc(&self, invoice: LnAddressPaymentRequest) -> anyhow::Result<()> {
        let payment_req = LndPaymentRequest::new(invoice.pr, 10, 150.to_string(), false);
        let mut lnd_ws = self.lightning_wallet.invoice_channel().await?;
        lnd_ws.sender.send(payment_req).await?;
        let mut event_stream = lnd_ws.event_stream::<LndPaymentResponse>();
        while let Some(LndWebsocketMessage::Response(payment_status)) = event_stream.next().await {
            info!("{:?}", payment_status);
            if payment_status.status() == InvoicePaymentState::Succeeded {
                let settled = self
                    .lightning_wallet
                    .settle_htlc(payment_status.preimage())
                    .await;
                info!("{:?}", settled);
                break;
            }
        }
        Ok(())
    }
}
