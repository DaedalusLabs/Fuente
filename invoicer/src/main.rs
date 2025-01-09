mod invoicer;
mod registries;
mod state;
mod uploads;

use anyhow::anyhow;
use fuente::models::{
    CommerceProfile, DriverProfile, OrderInvoiceState, OrderParticipant, OrderPaymentStatus,
    OrderRequest, OrderStatus, OrderUpdateRequest, ProductMenu, DRIVER_HUB_PUB_KEY,
    NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
    NOSTR_KIND_COMMERCE_UPDATE, NOSTR_KIND_CONSUMER_CANCEL, NOSTR_KIND_CONSUMER_ORDER_REQUEST,
    NOSTR_KIND_CONSUMER_REGISTRY, NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_COURIER_UPDATE,
    NOSTR_KIND_ORDER_STATE, NOSTR_KIND_PRESIGNED_URL_REQ, NOSTR_KIND_PRESIGNED_URL_RESP,
    NOSTR_KIND_SERVER_CONFIG, NOSTR_KIND_SERVER_REQUEST, TEST_PUB_KEY,
};
use invoicer::Invoicer;
use nostro2::{
    keypair::NostrKeypair,
    notes::NostrNote,
    relays::{NostrRelayPool, NostrSubscription, NoteEvent, PoolRelayBroadcaster, RelayEvent},
};
use state::InvoicerStateLock;
use upload_things::UtRecord;
use uploads::UtSigner;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let vec_strings = include_str!("relays.txt")
        .trim()
        .lines()
        .map(|x| x.trim().to_string())
        .collect();
    let relay_pool = NostrRelayPool::new(vec_strings).await?;
    let bot = InvoicerBot::new(relay_pool.writer.clone()).await?;
    tracing::info!("Bot created");
    if let Err(relay_future) = bot.read_relay_pool(relay_pool).await {
        tracing::error!("{:?}", relay_future);
    }
    Err(anyhow!("Bot ended"))
}

#[derive(Clone)]
pub struct InvoicerBot {
    server_keys: NostrKeypair,
    bot_state: InvoicerStateLock,
    broadcaster: PoolRelayBroadcaster,
    invoicer: Invoicer,
    uploader: UtSigner,
}

impl InvoicerBot {
    pub async fn new(broadcaster: PoolRelayBroadcaster) -> anyhow::Result<Self> {
        let server_keys = NostrKeypair::new(&std::env::var("FUENTE_PRIV_KEY")?)?;
        Ok(Self {
            invoicer: Invoicer::new().await?,
            server_keys,
            broadcaster,
            uploader: UtSigner::default(),
            bot_state: InvoicerStateLock::default(),
        })
    }
    pub async fn read_relay_pool(&self, mut relays: NostrRelayPool) -> anyhow::Result<()> {
        let mut live_filter = NostrSubscription {
            kinds: Some(vec![NOSTR_KIND_ORDER_STATE]),
            ..Default::default()
        };
        live_filter.add_tag("#p", DRIVER_HUB_PUB_KEY);
        let mut filter = NostrSubscription {
            kinds: Some(vec![
                NOSTR_KIND_SERVER_REQUEST,
                NOSTR_KIND_SERVER_CONFIG,
                NOSTR_KIND_ADMIN_REQUEST,
                NOSTR_KIND_PRESIGNED_URL_REQ,
                NOSTR_KIND_COURIER_PROFILE,
            ]),
            ..Default::default()
        };
        filter.add_tag("#p", TEST_PUB_KEY);
        let commerces_filter = NostrSubscription {
            kinds: Some(vec![
                NOSTR_KIND_COMMERCE_PROFILE,
                NOSTR_KIND_COMMERCE_PRODUCTS,
                NOSTR_KIND_CONSUMER_REGISTRY,
            ]),
            ..Default::default()
        };
        let config_filter = NostrSubscription {
            kinds: Some(vec![NOSTR_KIND_SERVER_CONFIG]),
            authors: Some(vec![TEST_PUB_KEY.to_string()]),
            ..Default::default()
        };
        relays.writer.subscribe(filter.relay_subscription()).await?;
        relays
            .writer
            .subscribe(commerces_filter.relay_subscription())
            .await?;
        relays
            .writer
            .subscribe(live_filter.relay_subscription())
            .await?;
        relays
            .writer
            .subscribe(config_filter.relay_subscription())
            .await?;
        while let Some(signed_note) = relays.listener.recv().await {
            if let RelayEvent::NewNote(NoteEvent(_, _, note)) = signed_note.1 {
                if let Err(e) = self.note_processor(note).await {
                    tracing::error!("{:?}", e);
                }
            }
        }
        Err(anyhow!("Relay pool closed"))
    }
    async fn note_processor(&self, signed_note: NostrNote) -> anyhow::Result<()> {
        match signed_note.kind {
            NOSTR_KIND_SERVER_REQUEST => {
                let decrypted = self.server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = self.handle_server_requests(inner_note, signed_note).await {
                    tracing::error!("{:?}", e);
                }
            }
            NOSTR_KIND_ADMIN_REQUEST => {
                let decrypted = self.server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                let update_note = self
                    .bot_state
                    .sign_updated_config(inner_note, &self.server_keys)
                    .await?;
                self.broadcaster.broadcast_note(update_note).await?;
            }
            NOSTR_KIND_CONSUMER_REGISTRY => {
                let decrypted = self.server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                self.bot_state.add_consumer_profile(inner_note).await?
            }
            NOSTR_KIND_ORDER_STATE => {
                let decrypted = self.server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                let _ = OrderInvoiceState::try_from(&inner_note)?;
                self.bot_state.update_live_order(inner_note).await?;
                tracing::info!("Order state updated");
            }
            _ => {
                if let Err(e) = self
                    .handle_public_notes(signed_note.kind, signed_note)
                    .await
                {
                    tracing::error!("{:?}", e);
                }
            }
        }
        Ok(())
    }
    async fn handle_public_notes(
        &self,
        note_kind: u32,
        signed_note: NostrNote,
    ) -> anyhow::Result<()> {
        match note_kind {
            NOSTR_KIND_COMMERCE_PROFILE => {
                CommerceProfile::try_from(signed_note.clone())?;
                self.bot_state.add_commerce_profile(signed_note).await?;
                tracing::info!("Added commerce profile");
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                ProductMenu::try_from(signed_note.clone())?;
                self.bot_state.add_commerce_menu(signed_note).await?;
                tracing::info!("Added menu");
            }
            NOSTR_KIND_COURIER_PROFILE => {
                let inner_note = self.server_keys.decrypt_nip_04_content(&signed_note)?;
                let driver_note = NostrNote::try_from(inner_note)?;
                DriverProfile::try_from(&driver_note)?;
                self.bot_state.add_courier_profile(driver_note).await?;
                tracing::info!("Added courier profile");
            }
            NOSTR_KIND_SERVER_CONFIG => {
                let decrypted = match self.server_keys.decrypt_nip_04_content(&signed_note) {
                    Ok(decrypted) => Some(decrypted),
                    Err(_e) => None,
                };
                self.bot_state
                    .update_admin_config(signed_note.clone(), decrypted)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
    async fn handle_server_requests(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        match inner_note.kind {
            NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                let order_req = OrderRequest::try_from(&inner_note)?;
                let registered_commerce = self
                    .bot_state
                    .find_commerce(order_req.commerce.as_str())
                    .await?;
                self.invoicer
                    .new_order_invoice(
                        order_req,
                        inner_note,
                        registered_commerce.0,
                        self.bot_state.exchange_rate().await,
                        self.server_keys.clone(),
                        self.bot_state.clone(),
                        self.broadcaster.clone(),
                    )
                    .await?;
            }
            NOSTR_KIND_CONSUMER_CANCEL => {
                let update_req = OrderUpdateRequest::try_from(inner_note)?;
                let mut invoice_state = update_req.invoice_state()?;
                if invoice_state.order.pubkey != outer_note.pubkey {
                    return Err(anyhow!("Unauthorized"));
                }
                let invoice = invoice_state
                    .commerce_invoice
                    .as_ref()
                    .cloned()
                    .ok_or(anyhow!("No invoice"))?;
                if let Err(e) = self.invoicer.cancel_htlc(invoice).await {
                    tracing::error!("{:?}", e);
                }
                invoice_state.order_status = OrderStatus::Canceled;
                let (update, giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Consumer, &self.server_keys)?;
                self.bot_state.update_live_order(update).await?;
                self.broadcaster.broadcast_note(giftwrap).await?;
                let (_, commerce_giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Commerce, &self.server_keys)?;
                self.broadcaster.broadcast_note(commerce_giftwrap).await?;
                tracing::info!("Order canceled");
            }
            NOSTR_KIND_COMMERCE_UPDATE => {
                self.handle_commerce_updates(inner_note, outer_note).await?;
            }
            NOSTR_KIND_COURIER_UPDATE => {
                self.handle_courier_order_update(inner_note, outer_note)
                    .await?;
            }
            NOSTR_KIND_PRESIGNED_URL_REQ => {
                if !self
                    .bot_state
                    .is_consumer_registered(&outer_note.pubkey)
                    .await
                    && !self
                        .bot_state
                        .is_commerce_whitelisted(&outer_note.pubkey)
                        .await
                {
                    tracing::error!("Unauthorized request {}", outer_note.pubkey);
                }
                if let Ok(presigned_url) = self.uploader.sign_url(inner_note.content.try_into()?) {
                    let ut_record = UtRecord {
                        file_keys: vec![presigned_url.file_key.clone()],
                        ..Default::default()
                    };
                    if let Ok(_) = self.uploader.register_url(ut_record).await {
                        let mut new_url_note = NostrNote {
                            kind: NOSTR_KIND_PRESIGNED_URL_RESP,
                            content: serde_json::to_string(&presigned_url)?,
                            pubkey: self.server_keys.public_key(),
                            ..Default::default()
                        };
                        self.server_keys
                            .sign_nip_04_encrypted(&mut new_url_note, outer_note.pubkey)?;
                        self.broadcaster.broadcast_note(new_url_note).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    async fn handle_commerce_updates(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        let commerce_update = OrderUpdateRequest::try_from(inner_note)?;
        let mut invoice_state = commerce_update.invoice_state()?;
        if commerce_update.order.pubkey != TEST_PUB_KEY {
            return Err(anyhow!("Not real order"));
        }
        if invoice_state.get_commerce_pubkey() != outer_note.pubkey {
            return Err(anyhow!("Unauthorized"));
        }
        match commerce_update.status_update {
            OrderStatus::Preparing => {
                let invoice = invoice_state
                    .commerce_invoice
                    .as_ref()
                    .cloned()
                    .ok_or(anyhow!("No invoice"))?;
                self.invoicer.settle_htlc(invoice).await?;
            }
            OrderStatus::ReadyForDelivery => {
                invoice_state.order_status = OrderStatus::ReadyForDelivery;
                let (update, giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Commerce, &self.server_keys)?;
                let (_, consumer_giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Consumer, &self.server_keys)?;
                let (_, courier_giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Courier, &self.server_keys)?;
                self.bot_state.update_live_order(update).await?;
                self.broadcaster.broadcast_note(giftwrap).await?;
                self.broadcaster.broadcast_note(consumer_giftwrap).await?;
                self.broadcaster.broadcast_note(courier_giftwrap).await?;
            }
            OrderStatus::Canceled => {
                invoice_state.order_status = OrderStatus::Canceled;
                let (update, giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Commerce, &self.server_keys)?;
                let (_, consumer_giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Consumer, &self.server_keys)?;
                let (_, courier_giftwrap) = invoice_state
                    .giftwrapped_order(OrderParticipant::Courier, &self.server_keys)?;
                self.bot_state.update_live_order(update).await?;
                self.broadcaster.broadcast_note(giftwrap).await?;
                self.broadcaster.broadcast_note(consumer_giftwrap).await?;
                self.broadcaster.broadcast_note(courier_giftwrap).await?;
            }
            _ => return Err(anyhow!("Invalid order status")),
        }
        Ok(())
    }
    async fn handle_courier_order_update(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        let order_state = OrderUpdateRequest::try_from(inner_note)?;
        let invoice_state = order_state.invoice_state()?;
        if order_state.order.pubkey != TEST_PUB_KEY {
            return Err(anyhow!("Not real order"));
        }
        let mut live_order = self
            .bot_state
            .find_live_order(invoice_state.order_id().as_str())
            .await
            .ok_or(anyhow!("Order not found"))?;
        let courier_profile = self
            .bot_state
            .find_whitelisted_courier(outer_note.pubkey.as_str())
            .await?;
        let has_driver_assigned = live_order.courier.is_some();
        if !has_driver_assigned {
            live_order.courier = Some(courier_profile);
            let (update, giftwrap) =
                live_order.giftwrapped_order(OrderParticipant::Courier, &self.server_keys)?;
            self.bot_state.update_live_order(update).await?;
            self.broadcaster.broadcast_note(giftwrap).await?;
            let (_, consumer_giftwrap) =
                live_order.giftwrapped_order(OrderParticipant::Consumer, &self.server_keys)?;
            self.broadcaster.broadcast_note(consumer_giftwrap).await?;
            let (_, commerce_giftwrap) =
                live_order.giftwrapped_order(OrderParticipant::Commerce, &self.server_keys)?;
            self.broadcaster.broadcast_note(commerce_giftwrap).await?;
            return Ok(());
        }
        match (&live_order.payment_status, &live_order.order_status) {
            (_, OrderStatus::Completed) => {}
            (_, OrderStatus::Canceled) => {}
            (OrderPaymentStatus::PaymentFailed, _) => {}
            _ => {
                live_order.order_status = order_state.status_update;
                let (update, giftwrap) =
                    live_order.giftwrapped_order(OrderParticipant::Courier, &self.server_keys)?;
                self.bot_state.update_live_order(update).await?;
                self.broadcaster.broadcast_note(giftwrap).await?;
                let (_, consumer_giftwrap) =
                    live_order.giftwrapped_order(OrderParticipant::Consumer, &self.server_keys)?;
                self.broadcaster.broadcast_note(consumer_giftwrap).await?;
                let (_, commerce_giftwrap) =
                    live_order.giftwrapped_order(OrderParticipant::Commerce, &self.server_keys)?;
                self.broadcaster.broadcast_note(commerce_giftwrap).await?;
                return Ok(());
            }
        }
        Err(anyhow!("Order state channel closed"))
    }
}
