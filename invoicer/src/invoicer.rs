use anyhow::anyhow;
use bright_lightning::{
    HodlState, InvoicePaymentState, LnAddressPaymentRequest, LndHodlInvoice, LndHodlInvoiceState,
    LndPaymentRequest, LndPaymentResponse, LndWebsocketMessage,
};
use fuente::models::{
    CommerceProfile, OrderInvoiceState, OrderParticipant, OrderPaymentStatus, OrderRequest,
    OrderStatus,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use tokio::sync::broadcast::Sender;

use crate::state::InvoicerStateLock;
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
        let invoice_total_srd = order.products.total();
        let invoice_satoshi_amount = invoice_total_srd / exchange_rate * SATOSHIS_IN_ONE_BTC;
        let invoice = commerce_profile
            .ln_address()
            .get_invoice(
                &self.rest_client,
                invoice_satoshi_amount as u64 * MILISATOSHIS_IN_ONE_SATOSHI,
            )
            .await?;
        let hodl_amount = invoice_satoshi_amount as u64 + ILLUMINODES_FEES + FUENTE_FEES;
        let hodl_invoice = self
            .lightning_wallet
            .get_hodl_invoice(invoice.r_hash()?, hodl_amount)
            .await?;
        Ok((invoice, hodl_invoice))
    }
    pub async fn order_payment_notifier(
        self,
        order_invoice: OrderInvoiceState,
        keys: NostrKeypair,
        state_clone: InvoicerStateLock,
        broadcaster: Sender<nostro2::relays::WebSocketMessage>,
    ) -> anyhow::Result<()> {
        let invoice = order_invoice
            .commerce_invoice
            .as_ref()
            .ok_or(anyhow!("No invoice"))?;
        let subscriber = self
            .lightning_wallet
            .subscribe_to_invoice(invoice.r_hash_url_safe()?)
            .await?;
        let iter = subscriber.receiver;
        let mut ping_counter = 0;
        while let Some(payment_response) = iter.read::<LndHodlInvoiceState>().await {
            match payment_response {
                LndWebsocketMessage::Response(invoice_state) => match invoice_state.state() {
                    HodlState::OPEN => {
                        let (signed_update, giftwrapped) =
                            order_invoice.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                        state_clone.update_live_order(signed_update).await?;
                        broadcaster.send(giftwrapped.into())?;
                    }
                    HodlState::ACCEPTED => {
                        let mut new_order = order_invoice.clone();
                        new_order.order_status = OrderStatus::Pending;
                        new_order.payment_status = OrderPaymentStatus::PaymentReceived;
                        let (signed_update, giftwrapped) =
                            new_order.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                        let (_, giftwrapped_commerce) =
                            new_order.giftwrapped_order(OrderParticipant::Commerce, &keys)?;
                        state_clone.update_live_order(signed_update).await?;
                        broadcaster.send(giftwrapped.into())?;
                        broadcaster.send(giftwrapped_commerce.into())?;
                    }
                    HodlState::SETTLED => {
                        let mut new_order = order_invoice.clone();
                        new_order.order_status = OrderStatus::Preparing;
                        new_order.payment_status = OrderPaymentStatus::PaymentSuccess;
                        let (signed_update, giftwrapped) =
                            new_order.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                        let (_, giftwrapped_commerce) =
                            new_order.giftwrapped_order(OrderParticipant::Commerce, &keys)?;
                        state_clone.update_live_order(signed_update).await?;
                        broadcaster.send(giftwrapped.into())?;
                        broadcaster.send(giftwrapped_commerce.into())?;
                        break;
                    }
                    HodlState::CANCELED => {
                        let mut new_order = order_invoice.clone();
                        new_order.order_status = OrderStatus::Canceled;
                        new_order.payment_status = OrderPaymentStatus::PaymentFailed;
                        let (_, giftwrapped) =
                            new_order.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                        let (_, giftwrapped_commerce) =
                            new_order.giftwrapped_order(OrderParticipant::Commerce, &keys)?;
                        state_clone
                            .remove_live_order(new_order.order_id().as_str())
                            .await?;
                        broadcaster.send(giftwrapped.into())?;
                        broadcaster.send(giftwrapped_commerce.into())?;
                        break;
                    }
                },
                LndWebsocketMessage::Error(_e) => {
                    self.cancel_htlc(invoice.clone()).await?;
                    let mut new_order = order_invoice.clone();
                    new_order.order_status = OrderStatus::Canceled;
                    new_order.payment_status = OrderPaymentStatus::PaymentFailed;
                    let (_, giftwrapped) =
                        new_order.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                    let (_, giftwrapped_commerce) =
                        new_order.giftwrapped_order(OrderParticipant::Commerce, &keys)?;
                    state_clone
                        .remove_live_order(new_order.order_id().as_str())
                        .await?;
                    broadcaster.send(giftwrapped.into())?;
                    broadcaster.send(giftwrapped_commerce.into())?;
                    break;
                }
                _ => {
                    ping_counter += 1;
                    if ping_counter > 5 {
                        tracing::warn!("Canceling HTLC due to inactivity");
                        self.cancel_htlc(invoice.clone()).await?;
                        let mut new_order = order_invoice.clone();
                        new_order.order_status = OrderStatus::Canceled;
                        new_order.payment_status = OrderPaymentStatus::PaymentFailed;
                        let (_, giftwrapped) =
                            new_order.giftwrapped_order(OrderParticipant::Consumer, &keys)?;
                        let (_, giftwrapped_commerce) =
                            new_order.giftwrapped_order(OrderParticipant::Commerce, &keys)?;
                        state_clone
                            .remove_live_order(new_order.order_id().as_str())
                            .await?;
                        broadcaster.send(giftwrapped.into())?;
                        broadcaster.send(giftwrapped_commerce.into())?;
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
        broadcaster: Sender<nostro2::relays::WebSocketMessage>,
    ) -> anyhow::Result<OrderInvoiceState> {
        let invoice = self
            .create_order_invoice(&order, &commerce, exchange_rate)
            .await?;
        let state_update = OrderInvoiceState::new(
            signed_note.clone(),
            Some(invoice.1),
            Some(invoice.0.clone()),
        );
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
        let payment_req = LndPaymentRequest::new(invoice.pr, 1000, 150.to_string(), false);
        let lnd_ws = self.lightning_wallet.invoice_channel().await?;
        lnd_ws.sender.send(payment_req).await?;
        let event_stream = lnd_ws.receiver;
        let mut ping_counter = 0;
        while let Some(ws_msg) = event_stream.read::<LndPaymentResponse>().await {
            match ws_msg {
                LndWebsocketMessage::Response(payment_status) => {
                    if payment_status.status() == InvoicePaymentState::Succeeded {
                        self.lightning_wallet
                            .settle_htlc(payment_status.preimage())
                            .await?;
                        break;
                    }
                }
                LndWebsocketMessage::Ping => {
                    ping_counter += 1;
                    if ping_counter > 5 {
                        break;
                    }
                }
                LndWebsocketMessage::Error(e) => {
                    tracing::error!("Error settling invoice {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}
