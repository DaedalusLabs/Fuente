use anyhow::anyhow;
use bright_lightning::{
    HodlState, InvoicePaymentState, LnAddressPaymentRequest, LndHodlInvoice, LndPaymentRequest,
};
use fuente::models::{
    CommerceProfile, OrderInvoiceState, OrderPaymentStatus, OrderRequest, OrderStatus,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote, relays::PoolRelayBroadcaster};
use tracing::{error, info, warn};
pub const SATOSHIS_IN_ONE_BTC: f64 = 100_000_000.0;
pub const MILISATOSHIS_IN_ONE_SATOSHI: u64 = 1000;
pub const ILLUMINODES_FEES: u64 = 20;
pub const FUENTE_FEES: u64 = 0;

#[derive(Clone)]
pub struct Invoicer {
    rest_client: reqwest::Client,
    lightning_wallet: bright_lightning::LightningClient,
    broadcaster: PoolRelayBroadcaster,
    keys: NostrKeypair,
}
impl Invoicer {
    async fn update_order_status(
        &self,
        state: &mut OrderInvoiceState,
        payment_status: OrderPaymentStatus,
        order_status: OrderStatus,
    ) -> anyhow::Result<()> {
        state.update_payment_status(payment_status.clone());
        state.update_order_status(order_status.clone());
        self.broadcast_order_update(&state).await?;
        Ok(())
    }
    async fn broadcast_order_update(&self, state: &OrderInvoiceState) -> anyhow::Result<()> {
        let user_note = state.sign_customer_update(&self.keys)?;
        let commerce_note = state.sign_commerce_update(&self.keys)?;
        self.broadcaster.broadcast_note(user_note).await?;
        self.broadcaster.broadcast_note(commerce_note).await?;
        match state.get_courier() {
            Some(_) => {
                let courier_note = state.sign_courier_update(&self.keys)?;
                self.broadcaster.broadcast_note(courier_note).await?;
            }
            None => {
                if state.get_order_status() == OrderStatus::ReadyForDelivery {
                    let courier_note = state.sign_courier_update(&self.keys)?;
                    self.broadcaster.broadcast_note(courier_note).await?;
                }
            }
        }
        Ok(())
    }
    pub async fn new(
        broadcaster: PoolRelayBroadcaster,
        keys: NostrKeypair,
    ) -> anyhow::Result<Self> {
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
            broadcaster,
            keys,
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
    async fn order_payment_notifier(self, mut state: OrderInvoiceState) -> anyhow::Result<bool> {
        let invoice = state
            .get_commerce_invoice()
            .ok_or(anyhow!("No commerce invoice found"))?;
        let mut subscriber = self
            .lightning_wallet
            .subscribe_to_invoice(invoice.r_hash_url_safe()?)
            .await?;
        let mut success = false;
        loop {
            if subscriber.receiver.is_closed() {
                self.lightning_wallet
                    .cancel_htlc(invoice.r_hash_url_safe()?)
                    .await?;
                warn!("Canceled HTLC due to inactivity");
                self.update_order_status(
                    &mut state,
                    OrderPaymentStatus::PaymentFailed,
                    OrderStatus::Canceled,
                )
                .await?;
                break;
            }
            if let Some(status) = subscriber.receiver.recv().await {
                match status.state() {
                    HodlState::ACCEPTED => {
                        self.update_order_status(
                            &mut state,
                            OrderPaymentStatus::PaymentReceived,
                            OrderStatus::Pending,
                        )
                        .await?;
                        info!("Payment received");
                    }
                    HodlState::CANCELED => {
                        self.lightning_wallet
                            .cancel_htlc(invoice.r_hash_url_safe()?)
                            .await?;
                        self.update_order_status(
                            &mut state,
                            OrderPaymentStatus::PaymentFailed,
                            OrderStatus::Canceled,
                        )
                        .await?;
                        warn!("Payment canceled");
                        break;
                    }
                    HodlState::SETTLED => {
                        self.update_order_status(
                            &mut state,
                            OrderPaymentStatus::PaymentSuccess,
                            OrderStatus::Preparing,
                        )
                        .await?;
                        success = true;
                        info!("Order successfully paid");
                        break;
                    }
                    HodlState::OPEN => {
                        continue;
                    }
                }
            }
        }
        Ok(success)
    }
    pub async fn handle_order_request(
        &self,
        order: OrderRequest,
        signed_note: NostrNote,
        commerce: CommerceProfile,
        exchange_rate: f64,
        keys: &NostrKeypair,
    ) -> anyhow::Result<NostrNote> {
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
        let state_update_note = state_update.sign_customer_update(keys)?;
        info!("Order state update signed");
        let self_clone = self.clone();
        tokio::spawn(async move {
            if let Err(e) = self_clone.order_payment_notifier(state_update).await {
                error!("NOTIFIER ERROR {:?}", e);
            }
        });
        Ok(state_update_note)
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
        lnd_ws.sender.send(payment_req)?;
        loop {
            if lnd_ws.receiver.is_closed() && lnd_ws.receiver.is_empty() {
                return Err(anyhow!("Payment channel closed"));
            }
            if let Some(payment_status) = lnd_ws.receiver.recv().await {
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
        }
        Ok(())
    }
}
