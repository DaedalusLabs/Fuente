mod invoicer;
mod registries;
mod state;
mod uploads;

const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];

use anyhow::anyhow;
use fuente::models::{
    AdminServerRequest, CommerceProfile, DriverProfile, OrderInvoiceState, OrderParticipant,
    OrderPaymentStatus, OrderRequest, OrderStatus, OrderUpdateRequest, ProductMenu,
    DRIVER_HUB_PUB_KEY, NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_COMMERCE_PRODUCTS,
    NOSTR_KIND_COMMERCE_PROFILE, NOSTR_KIND_COMMERCE_UPDATE, NOSTR_KIND_CONSUMER_ORDER_REQUEST,
    NOSTR_KIND_CONSUMER_REGISTRY, NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_COURIER_UPDATE,
    NOSTR_KIND_ORDER_STATE, NOSTR_KIND_PRESIGNED_URL_REQ, NOSTR_KIND_PRESIGNED_URL_RESP,
    NOSTR_KIND_SERVER_CONFIG, NOSTR_KIND_SERVER_REQUEST, TEST_PRIV_KEY, TEST_PUB_KEY,
};
use invoicer::Invoicer;
use nostro2::{
    keypair::NostrKeypair,
    notes::NostrNote,
    relays::{NostrRelayPool, NostrSubscription, NoteEvent, PoolRelayBroadcaster, RelayEvent},
};
use state::InvoicerStateLock;
use tracing::{error, info, Level};
use upload_things::UtRecord;
use uploads::UtSigner;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();
    let vec_strings = RELAY_URLS.iter().map(|s| s.to_string()).collect();
    let relay_pool = NostrRelayPool::new(vec_strings).await?;
    let bot = InvoicerBot::new(relay_pool.writer.clone()).await?;
    info!("Bot created");
    let relay_future = bot.read_relay_pool(relay_pool, bot.keys.clone());
    loop {
        tokio::select! {
            _ = relay_future => {
                error!("Relay pool task ended");
                break;
            }
        }
    }
    Err(anyhow!("Bot ended"))
}

#[derive(Clone)]
pub struct InvoicerBot {
    keys: NostrKeypair,
    bot_state: InvoicerStateLock,
    broadcaster: PoolRelayBroadcaster,
    invoicer: Invoicer,
    uploader: UtSigner,
}

impl InvoicerBot {
    pub async fn new(broadcaster: PoolRelayBroadcaster) -> anyhow::Result<Self> {
        // TODO
        // make this env variable
        let server_keys = NostrKeypair::new(TEST_PRIV_KEY)?;
        info!("Relay pool started");
        Ok(Self {
            invoicer: Invoicer::new().await?,
            keys: server_keys,
            broadcaster,
            uploader: UtSigner::default(),
            bot_state: InvoicerStateLock::default(),
        })
    }
    async fn broadcast_order_update(
        broadcaster: PoolRelayBroadcaster,
        keys: NostrKeypair,
        state: &OrderInvoiceState,
    ) -> anyhow::Result<()> {
        let user_note = state.sign_update_for(OrderParticipant::Consumer, &keys)?;
        let commerce_note = state.sign_update_for(OrderParticipant::Commerce, &keys)?;
        broadcaster.broadcast_note(user_note).await?;
        broadcaster.broadcast_note(commerce_note).await?;
        match state.courier {
            Some(_) => {
                let courier_note = state.sign_update_for(OrderParticipant::Courier, &keys)?;
                broadcaster.broadcast_note(courier_note).await?;
            }
            None => {
                if state.order_status == OrderStatus::ReadyForDelivery {
                    let courier_note = state.sign_update_for(OrderParticipant::Courier, &keys)?;
                    broadcaster.broadcast_note(courier_note).await?;
                }
            }
        }
        Ok(())
    }
    pub async fn read_relay_pool(
        &self,
        mut relays: NostrRelayPool,
        keys: NostrKeypair,
    ) -> anyhow::Result<()> {
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
            ]),
            ..Default::default()
        };
        filter.add_tag("#p", TEST_PUB_KEY);
        let commerces_filter = NostrSubscription {
            kinds: Some(vec![
                NOSTR_KIND_COMMERCE_PROFILE,
                NOSTR_KIND_COMMERCE_PRODUCTS,
                NOSTR_KIND_COURIER_PROFILE,
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
        let server_keys = keys.clone();
        while let Some(signed_note) = relays.listener.recv().await {
            if let RelayEvent::NewNote(NoteEvent(_, _, note)) = signed_note.1 {
                if let Err(e) = self.note_processor(server_keys.clone(), note).await {
                    error!("{:?}", e);
                }
            }
        }
        Err(anyhow!("Relay pool closed"))
    }
    async fn note_processor(
        &self,
        server_keys: NostrKeypair,
        signed_note: NostrNote,
    ) -> anyhow::Result<()> {
        match signed_note.kind {
            NOSTR_KIND_SERVER_REQUEST => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = self.handle_private_notes(inner_note.kind, inner_note).await {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_ADMIN_REQUEST => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = self.handle_private_notes(inner_note.kind, inner_note).await {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_CONSUMER_REGISTRY => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = self.handle_private_notes(inner_note.kind, inner_note).await {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_SERVER_CONFIG => {
                if let Err(e) = self
                    .handle_public_notes(signed_note.kind, signed_note)
                    .await
                {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_COMMERCE_PROFILE => {
                if let Err(e) = self
                    .handle_public_notes(signed_note.kind, signed_note)
                    .await
                {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                if let Err(e) = self
                    .handle_public_notes(signed_note.kind, signed_note)
                    .await
                {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_COURIER_PROFILE => {
                if let Err(e) = self
                    .handle_public_notes(signed_note.kind, signed_note)
                    .await
                {
                    error!("{:?}", e);
                }
            }
            _ => {}
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
                info!("Added commerce profile");
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                ProductMenu::try_from(signed_note.clone())?;
                self.bot_state.add_commerce_menu(signed_note).await?;
                info!("Added menu");
            }
            NOSTR_KIND_COURIER_PROFILE => {
                DriverProfile::try_from(signed_note.clone())?;
                self.bot_state
                    .check_courier_whitelist(signed_note.pubkey.as_str())
                    .await?;
                self.bot_state.add_courier_profile(signed_note).await?;
                info!("Added courier profile");
            }
            NOSTR_KIND_SERVER_CONFIG => {
                let decrypted = match self.keys.decrypt_nip_04_content(&signed_note) {
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
    async fn handle_private_notes(
        &self,
        note_kind: u32,
        signed_note: NostrNote,
    ) -> anyhow::Result<()> {
        match note_kind {
            NOSTR_KIND_PRESIGNED_URL_REQ => {
                if !self
                    .bot_state
                    .is_consumer_registered(&signed_note.pubkey)
                    .await
                    && !self
                        .bot_state
                        .is_commerce_whitelisted(&signed_note.pubkey)
                        .await
                {
                    error!("Unauthorized request");
                }
                if let Ok(presigned_url) = self.uploader.sign_url(signed_note.content.try_into()?) {
                    let ut_record = UtRecord {
                        file_keys: vec![presigned_url.file_key.clone()],
                        ..Default::default()
                    };
                    if let Ok(_) = self.uploader.register_url(ut_record).await {
                        let mut new_url_note = NostrNote {
                            kind: NOSTR_KIND_PRESIGNED_URL_RESP,
                            content: serde_json::to_string(&presigned_url)?,
                            pubkey: self.keys.public_key(),
                            ..Default::default()
                        };
                        self.keys
                            .sign_nip_04_encrypted(&mut new_url_note, signed_note.pubkey)?;
                        self.broadcaster.broadcast_note(new_url_note).await?;
                    }
                }
            }
            NOSTR_KIND_CONSUMER_REGISTRY => {
                self.bot_state.add_consumer_profile(signed_note).await?
            }
            NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                let order_req = OrderRequest::try_from(&signed_note)?;
                let registered_commerce = self
                    .bot_state
                    .find_commerce(order_req.commerce.as_str())
                    .await?;
                self.invoicer
                    .new_order_invoice(
                        order_req,
                        signed_note,
                        registered_commerce.0,
                        self.bot_state.exchange_rate().await,
                        self.keys.clone(),
                        self.bot_state.clone(),
                        self.broadcaster.clone(),
                    )
                    .await?;
            }
            NOSTR_KIND_COMMERCE_UPDATE => {
                self.handle_commerce_updates(signed_note.clone(), signed_note.clone())
                    .await?;
            }
            NOSTR_KIND_COURIER_UPDATE => {
                self.handle_courier_order_update(signed_note.clone(), signed_note.clone())
                    .await?;
            }
            NOSTR_KIND_ADMIN_REQUEST => {
                if let Ok(admin_req) = AdminServerRequest::try_from(&signed_note) {
                    let update_note = self
                        .bot_state
                        .sign_updated_config(admin_req, &self.keys)
                        .await?;
                    self.broadcaster.broadcast_note(update_note).await?;
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
        let mut live_order = self
            .bot_state
            .find_live_order(commerce_update.order_id.as_str())
            .await
            .ok_or(anyhow!("Order not found"))?;
        let commerce_pubkey = live_order.get_commerce_pubkey();
        if commerce_pubkey != outer_note.pubkey {
            return Err(anyhow!("Unauthorized"));
        }
        match commerce_update.status_update {
            OrderStatus::Preparing => {
                live_order.order_status = OrderStatus::Preparing;
                self.invoicer
                    .settle_htlc(
                        live_order
                            .commerce_invoice
                            .as_ref()
                            .cloned()
                            .ok_or(anyhow!("No commerce invoice"))?,
                    )
                    .await?;
                self.bot_state.update_live_order(live_order.clone()).await?;
                InvoicerBot::broadcast_order_update(
                    self.broadcaster.clone(),
                    self.keys.clone(),
                    &live_order,
                )
                .await?;
                Ok(())
            }
            OrderStatus::ReadyForDelivery => {
                live_order.order_status = OrderStatus::ReadyForDelivery;
                self.bot_state.update_live_order(live_order.clone()).await?;
                InvoicerBot::broadcast_order_update(
                    self.broadcaster.clone(),
                    self.keys.clone(),
                    &live_order,
                )
                .await?;
                Ok(())
            }
            _ => Err(anyhow!("Invalid order status")),
        }
    }
    async fn handle_courier_order_update(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        let order_state = OrderUpdateRequest::try_from(inner_note)?;
        let mut live_order = self
            .bot_state
            .find_live_order(order_state.order_id.as_str())
            .await
            .ok_or(anyhow!("Order not found"))?;
        let courier_profile = self
            .bot_state
            .check_courier_whitelist(outer_note.pubkey.as_str())
            .await?;
        let has_driver_assigned = live_order.courier.is_some();
        if !has_driver_assigned {
            live_order.courier = Some(courier_profile);
            self.bot_state.update_live_order(live_order.clone()).await?;
            InvoicerBot::broadcast_order_update(
                self.broadcaster.clone(),
                self.keys.clone(),
                &live_order,
            )
            .await?;
            return Ok(());
        }
        match (&live_order.payment_status, &live_order.order_status) {
            (_, OrderStatus::Completed) => {}
            (_, OrderStatus::Canceled) => {}
            (OrderPaymentStatus::PaymentFailed, _) => {}
            _ => {
                live_order.order_status = order_state.status_update;
                self.bot_state.update_live_order(live_order.clone()).await?;
                InvoicerBot::broadcast_order_update(
                    self.broadcaster.clone(),
                    self.keys.clone(),
                    &live_order,
                )
                .await?;
                return Ok(());
            }
        }
        Err(anyhow!("Order state channel closed"))
    }
}
