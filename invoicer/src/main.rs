mod invoicer;
mod registries;
mod state;
mod uploads;

const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];

use anyhow::anyhow;
use async_channel::{Receiver, Sender};
use fuente::models::{
    AdminServerRequest, CommerceProfile, DriverProfile, OrderInvoiceState, OrderPaymentStatus,
    OrderRequest, OrderStatus, ProductMenu, DRIVER_HUB_PRIV_KEY, DRIVER_HUB_PUB_KEY,
    NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
    NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_CONSUMER_REGISTRY, NOSTR_KIND_COURIER_PROFILE,
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
use tracing::{error, info, warn, Level};
use upload_things::UtRecord;
use uploads::UtSigner;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();
    let vec_strings = RELAY_URLS.iter().map(|s| s.to_string()).collect();
    let relay_pool = NostrRelayPool::new(vec_strings).await?;
    let (public_notes_tx, public_notes_channel) = async_channel::unbounded();
    let (private_notes_tx, private_notes_channel) = async_channel::unbounded();
    let (live_order_tx, live_order_channel) = async_channel::unbounded();
    let bot = InvoicerBot::new(
        relay_pool.writer.clone(),
        public_notes_channel,
        private_notes_channel,
        live_order_channel,
    )
    .await?;
    info!("Bot created");
    let relay_future = InvoicerBot::read_relay_pool(
        relay_pool,
        bot.keys.clone(),
        public_notes_tx.clone(),
        private_notes_tx.clone(),
        live_order_tx.clone(),
    );
    let public_notes_future = bot.handle_public_notes(bot.public_notes_channel.clone());
    let private_notes_future = bot.handle_private_notes(bot.private_notes_channel.clone());
    let order_states_future = bot.handle_courier_order_update(bot.live_order_channel.clone());
    loop {
        tokio::select! {
            _ = relay_future => {
                error!("Relay pool task ended");
                break;
            }
             _ = public_notes_future => {
                 error!("Public channel task ended");
                 break;
             }
             _ = private_notes_future => {
                 error!("Private notes channel task ended");
                 break;
             }
             _ = order_states_future => {
                 error!("Order state task ended");
                 break;
             }
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct InvoicerBot {
    keys: NostrKeypair,
    invoicer: Invoicer,
    broadcaster: PoolRelayBroadcaster,
    uploader: UtSigner,
    bot_state: InvoicerStateLock,
    public_notes_channel: Receiver<(u32, NostrNote)>,
    private_notes_channel: Receiver<(u32, NostrNote)>,
    live_order_channel: Receiver<OrderInvoiceState>,
}

impl InvoicerBot {
    pub async fn new(
        broadcaster: PoolRelayBroadcaster,
        public_notes_channel: Receiver<(u32, NostrNote)>,
        private_notes_channel: Receiver<(u32, NostrNote)>,
        live_order_channel: Receiver<OrderInvoiceState>,
    ) -> anyhow::Result<Self> {
        // TODO
        // make this env variable
        let server_keys = NostrKeypair::new(TEST_PRIV_KEY)?;
        info!("Relay pool started");
        Ok(Self {
            invoicer: Invoicer::new(broadcaster.clone(), server_keys.clone()).await?,
            keys: server_keys,
            broadcaster,
            uploader: UtSigner::default(),
            bot_state: InvoicerStateLock::default(),
            public_notes_channel,
            private_notes_channel,
            live_order_channel,
        })
    }
    async fn note_processor(
        server_keys: NostrKeypair,
        courier_hub_keys: NostrKeypair,
        commerce_profile_channel: Sender<(u32, NostrNote)>,
        private_notes_channel: Sender<(u32, NostrNote)>,
        live_order_channel: Sender<OrderInvoiceState>,
        signed_note: NostrNote,
    ) -> anyhow::Result<()> {
        match signed_note.kind {
            NOSTR_KIND_SERVER_REQUEST => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = private_notes_channel
                    .send((inner_note.kind, inner_note))
                    .await
                {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_ADMIN_REQUEST => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = private_notes_channel
                    .send((signed_note.kind, inner_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_CONSUMER_REGISTRY => {
                let decrypted = server_keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = NostrNote::try_from(decrypted)?;
                if let Err(e) = private_notes_channel
                    .send((signed_note.kind, inner_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_SERVER_CONFIG => {
                if let Err(e) = commerce_profile_channel
                    .send((signed_note.kind, signed_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_ORDER_STATE => {
                let decrypted = courier_hub_keys.decrypt_nip_04_content(&signed_note)?;
                if let Err(e) = live_order_channel
                    .send(OrderInvoiceState::try_from(decrypted)?)
                    .await
                {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_COMMERCE_PROFILE => {
                if let Err(e) = commerce_profile_channel
                    .send((signed_note.kind, signed_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                if let Err(e) = commerce_profile_channel
                    .send((signed_note.kind, signed_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_COURIER_PROFILE => {
                if let Err(e) = commerce_profile_channel
                    .send((signed_note.kind, signed_note))
                    .await
                {
                    error!("{:?}", e);
                };
            }
            NOSTR_KIND_PRESIGNED_URL_REQ => {
                info!("Received presigned URL request");
                if let Err(e) = private_notes_channel
                    .send((signed_note.kind, signed_note))
                    .await
                {
                    error!("Failed to process presigned URL request: {:?}", e);
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn read_relay_pool(
        mut relays: NostrRelayPool,
        keys: NostrKeypair,
        commerce_profile_channel: Sender<(u32, NostrNote)>,
        private_notes_channel: Sender<(u32, NostrNote)>,
        live_order_channel: Sender<OrderInvoiceState>,
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
                NOSTR_KIND_PRESIGNED_URL_REQ,
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
        let courier_keys = NostrKeypair::new(DRIVER_HUB_PRIV_KEY)?;
        while let Some(signed_note) = relays.listener.recv().await {
            if let RelayEvent::NewNote(NoteEvent(_, _, note)) = signed_note.1 {
                if let Err(e) = InvoicerBot::note_processor(
                    server_keys.clone(),
                    courier_keys.clone(),
                    commerce_profile_channel.clone(),
                    private_notes_channel.clone(),
                    live_order_channel.clone(),
                    note,
                )
                .await
                {
                    error!("{:?}", e);
                }
            }
        }
        Err(anyhow!("Relay pool closed"))
    }
    async fn handle_public_notes(
        &self,
        note_channel: Receiver<(u32, NostrNote)>,
    ) -> anyhow::Result<()> {
        loop {
            if note_channel.is_closed() {
                error!("Public notes channel closed");
                break;
            }
            if let Ok((note_kind, signed_note)) = note_channel.recv().await {
                match note_kind {
                    NOSTR_KIND_COMMERCE_PROFILE => {
                        match CommerceProfile::try_from(signed_note.clone()) {
                            Ok(_) => {
                                if let Err(e) =
                                    self.bot_state.add_commerce_profile(signed_note).await
                                {
                                    error!("{:?}", e);
                                }
                                info!("Added commerce profile");
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                continue;
                            }
                        }
                    }
                    NOSTR_KIND_COMMERCE_PRODUCTS => {
                        match ProductMenu::try_from(signed_note.clone()) {
                            Ok(_) => {
                                if let Err(e) = self.bot_state.add_commerce_menu(signed_note).await
                                {
                                    error!("{:?}", e);
                                }
                                info!("Added menu");
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                continue;
                            }
                        }
                    }
                    NOSTR_KIND_COURIER_PROFILE => {
                        match DriverProfile::try_from(signed_note.clone()) {
                            Ok(_) => {
                                if let Err(e) = self
                                    .bot_state
                                    .check_courier_whitelist(signed_note.pubkey.as_str())
                                    .await
                                {
                                    error!("{:?}", e);
                                }
                                if let Err(e) =
                                    self.bot_state.add_courier_profile(signed_note).await
                                {
                                    error!("{:?}", e);
                                }
                                info!("Added courier profile");
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                continue;
                            }
                        }
                    }
                    NOSTR_KIND_SERVER_CONFIG => {
                        let decrypted = match self.keys.decrypt_nip_04_content(&signed_note) {
                            Ok(decrypted) => Some(decrypted),
                            Err(_e) => None,
                        };
                        if let Err(e) = self
                            .bot_state
                            .update_admin_config(signed_note.clone(), decrypted)
                            .await
                        {
                            error!("{:?}", e);
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(anyhow!("Public notes channel closed"))
    }
    async fn handle_private_notes(
        &self,
        note_channel: Receiver<(u32, NostrNote)>,
    ) -> anyhow::Result<()> {
        loop {
            if note_channel.is_closed() {
                break;
            }
            if let Ok((note_kind, signed_note)) = note_channel.recv().await {
                let process: anyhow::Result<()> = {
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
                                continue;
                            }
                            if let Ok(presigned_url) =
                                self.uploader.sign_url(signed_note.content.try_into()?)
                            {
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
                                    self.keys.sign_nip_04_encrypted(
                                        &mut new_url_note,
                                        signed_note.pubkey,
                                    )?;
                                    self.broadcaster.broadcast_note(new_url_note).await?;
                                }
                            }
                        }
                        NOSTR_KIND_CONSUMER_REGISTRY => {
                            if let Err(e) = self.bot_state.add_consumer_profile(signed_note).await {
                                error!("{:?}", e);
                            }
                            info!("Added consumer profile");
                        }
                        NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                            if let Ok(order_req) = OrderRequest::try_from(&signed_note) {
                                match self
                                    .bot_state
                                    .find_commerce(order_req.commerce.as_str())
                                    .await
                                {
                                    Ok(registered_commerce) => {
                                        if let Ok(update) = self
                                            .invoicer
                                            .handle_order_request(
                                                order_req,
                                                signed_note,
                                                registered_commerce.0,
                                                self.bot_state.exchange_rate().await,
                                                &self.keys,
                                            )
                                            .await
                                        {
                                            self.broadcaster.broadcast_note(update).await?;
                                        }
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                        continue;
                                    }
                                }
                            }
                        }
                        NOSTR_KIND_ORDER_STATE => {
                            if let Err(e) = self.handle_order_state_update(signed_note).await {
                                error!("ORDER STATE ERROR {:?}", e);
                            }
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
                };
                if let Err(e) = process {
                    error!(" PRIVATE NOTE ERRORED WITH {:?}", e);
                }
            }
        }
        Err(anyhow!("Private notes channel closed"))
    }
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
    async fn handle_commerce_updates(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        let mut order: OrderInvoiceState =
            OrderInvoiceState::try_from(inner_note.content.to_string())?;
        if outer_note.pubkey != order.get_order_request().commerce {
            return Err(anyhow!("Only commerce can update order"));
        }
        match order.get_order_status() {
            OrderStatus::Preparing => {
                let invoice = order
                    .get_commerce_invoice()
                    .ok_or(anyhow!("No invoice found"))?;
                let _ = self.invoicer.settle_htlc(invoice).await?;
            }
            OrderStatus::ReadyForDelivery => {
                let _ = self
                    .update_order_status(
                        &mut order,
                        OrderPaymentStatus::PaymentSuccess,
                        OrderStatus::ReadyForDelivery,
                    )
                    .await?;
                self.broadcast_order_update(&order).await?;
                self.bot_state.add_live_order(order).await?;
            }
            OrderStatus::Canceled => {
                if let Some(invoice) = order.get_commerce_invoice() {
                    if let Err(e) = self.invoicer.cancel_htlc(invoice).await {
                        warn!("{:?}", e);
                    }
                }
                let _ = self
                    .update_order_status(
                        &mut order,
                        OrderPaymentStatus::PaymentFailed,
                        OrderStatus::Canceled,
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
    async fn handle_courier_order_update(
        &self,
        order_state: Receiver<OrderInvoiceState>,
    ) -> anyhow::Result<()> {
        loop {
            if order_state.is_closed() {
                break;
            }
            if let Ok(order_state) = order_state.recv().await {
                match (
                    order_state.get_payment_status(),
                    order_state.get_order_status(),
                ) {
                    (_, OrderStatus::Completed) => {}
                    (_, OrderStatus::Canceled) => {}
                    (OrderPaymentStatus::PaymentFailed, _) => {}
                    _ => {
                        if let Err(e) = self.bot_state.add_live_order(order_state).await {
                            error!("{:?}", e);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Order state channel closed"))
    }
    async fn handle_order_state_update(&self, signed_note: NostrNote) -> anyhow::Result<()> {
        let mut order_state = OrderInvoiceState::try_from(signed_note.content.clone())?;
        let commerce = order_state.get_order_request().commerce.clone();
        if signed_note.pubkey == commerce {
            if let Err(e) = self
                .handle_commerce_updates(signed_note.clone(), signed_note.clone())
                .await
            {
                error!("{:?}", e);
            }
        }
        if signed_note.pubkey == order_state.get_order().pubkey {
            if order_state.get_order_status() == OrderStatus::Canceled {
                let invoice = order_state
                    .get_commerce_invoice()
                    .ok_or(anyhow!("No commerce invoice found"))?;
                self.invoicer.cancel_htlc(invoice).await?;
                warn!("Canceled HTLC due to inactivity");
                self.update_order_status(
                    &mut order_state,
                    OrderPaymentStatus::PaymentFailed,
                    OrderStatus::Canceled,
                )
                .await?;
            }
        }
        match order_state.get_courier() {
            Some(courier_note) => {
                info!("Courier assigned");
                if signed_note.pubkey == courier_note.pubkey {
                    info!("Courier update received");
                    let mut new_state = self
                        .bot_state
                        .handle_courier_updates(signed_note.clone(), signed_note.clone())
                        .await?;
                    let payment_status = new_state.get_payment_status();
                    let order_status = new_state.get_order_status();
                    if let Err(e) = self
                        .update_order_status(
                            &mut new_state,
                            payment_status.clone(),
                            order_status.clone(),
                        )
                        .await
                    {
                        error!("{:?}", e);
                    }
                }
            }
            None => {
                if order_state.get_order_status() == OrderStatus::ReadyForDelivery {
                    info!("Order ready for delivery");
                    let mut new_state = self
                        .bot_state
                        .handle_courier_updates(signed_note.clone(), signed_note.clone())
                        .await?;
                    info!("Courier assigned");
                    let payment_status = new_state.get_payment_status();
                    let order_status = new_state.get_order_status();
                    if let Err(e) = self
                        .update_order_status(
                            &mut new_state,
                            payment_status.clone(),
                            order_status.clone(),
                        )
                        .await
                    {
                        error!("{:?}", e);
                    }
                }
            }
        }
        info!("Order state updated");
        Ok(())
    }
}
