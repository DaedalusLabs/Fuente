const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use async_channel::{Receiver, Sender};
use bright_lightning::{
    HodlState, InvoicePaymentState, LightningAddress, LightningClient, LnAddressPaymentRequest,
    LndHodlInvoice, LndPaymentRequest,
};
use fuente::models::{
    CommerceProfile, DriverProfile, ProductMenu, DRIVER_HUB_PRIV_KEY, DRIVER_HUB_PUB_KEY,
    TEST_PRIV_KEY, TEST_PUB_KEY, {AdminConfiguration, AdminConfigurationType, AdminServerRequest},
    {OrderInvoiceState, OrderPaymentStatus, OrderRequest, OrderStatus},
    {
        NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
        NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_ORDER_STATE,
        NOSTR_KIND_SERVER_CONFIG, NOSTR_KIND_SERVER_REQUEST,
    },
};
use nostro2::{
    keypair::NostrKeypair,
    notes::{NostrNote, NostrTag},
    relays::{NostrRelayPool, NostrSubscription, NoteEvent, PoolRelayBroadcaster, RelayEvent},
};
use tokio::sync::Mutex;
use tracing::{error, info, warn, Level};
pub const SATOSHIS_IN_ONE_BTC: f64 = 100_000_000.0;
pub const MILISATOSHIS_IN_ONE_SATOSHI: u64 = 1000;
pub const ILLUMINODES_FEES: u64 = 20;
pub const FUENTE_FEES: u64 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommerceRegistryEntry {
    profile: Option<NostrNote>,
    menu: Option<NostrNote>,
    whitelisted: bool,
}
impl CommerceRegistryEntry {
    pub fn new(profile: Option<NostrNote>, menu: Option<NostrNote>) -> Self {
        Self {
            profile,
            menu,
            whitelisted: false,
        }
    }
    pub fn set_whitelisted(&mut self, whitelisted: bool) {
        self.whitelisted = whitelisted;
    }
    pub fn is_whitelisted(&self) -> bool {
        self.whitelisted
    }
    pub fn get_profile(&self) -> Option<NostrNote> {
        self.profile.clone()
    }
    pub fn get_menu(&self) -> Option<NostrNote> {
        self.menu.clone()
    }
    pub fn set_profile(&mut self, profile: NostrNote) {
        self.profile = Some(profile);
    }
    pub fn set_menu(&mut self, menu: NostrNote) {
        self.menu = Some(menu);
    }
    pub fn ln_address(&self) -> anyhow::Result<LightningAddress> {
        let profile = self.profile.clone().ok_or(anyhow!("No profile found"))?;
        let commerce_profile = CommerceProfile::try_from(profile)?;
        Ok(commerce_profile.ln_address())
    }
}

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
    ln_address_client: reqwest::Client,
    lightning_wallet: LightningClient,
    broadcaster: PoolRelayBroadcaster,
    admin_config: Arc<Mutex<AdminConfiguration>>,
    live_orders: Arc<Mutex<HashMap<String, OrderInvoiceState>>>,
    courier_profiles: Arc<Mutex<HashMap<String, NostrNote>>>,
    commerce_registries: Arc<Mutex<HashMap<String, CommerceRegistryEntry>>>,
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
        let lightning_wallet =
            LightningClient::new("lnd.illuminodes.com", "./invoices.macaroon").await?;
        let server_keys = NostrKeypair::new(TEST_PRIV_KEY)?;
        info!("Relay pool started");
        let mut admin_config = AdminConfiguration::default();
        admin_config.set_admin_whitelist(vec![TEST_PUB_KEY.to_string()]);
        Ok(Self {
            keys: server_keys,
            broadcaster,
            ln_address_client: reqwest::Client::new(),
            admin_config: Arc::new(Mutex::new(admin_config)),
            courier_profiles: Arc::new(Mutex::new(HashMap::new())),
            commerce_registries: Arc::new(Mutex::new(HashMap::new())),
            live_orders: Arc::new(Mutex::new(HashMap::new())),
            lightning_wallet,
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
            ]),
            ..Default::default()
        };
        filter.add_tag("#p", TEST_PUB_KEY);
        let commerces_filter = NostrSubscription {
            kinds: Some(vec![
                NOSTR_KIND_COMMERCE_PROFILE,
                NOSTR_KIND_COMMERCE_PRODUCTS,
                NOSTR_KIND_COURIER_PROFILE,
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
                break;
            }
            if let Ok((note_kind, signed_note)) = note_channel.recv().await {
                match note_kind {
                    NOSTR_KIND_COMMERCE_PROFILE => {
                        if let Ok(_) = CommerceProfile::try_from(signed_note.clone()) {
                            let mut profiles = self.commerce_registries.lock().await;
                            let entry = profiles.entry(signed_note.pubkey.to_string()).or_insert(
                                CommerceRegistryEntry {
                                    profile: None,
                                    menu: None,
                                    whitelisted: false,
                                },
                            );
                            entry.set_profile(signed_note.clone());
                            info!("Added commerce profile");
                        }
                    }
                    NOSTR_KIND_COMMERCE_PRODUCTS => {
                        ProductMenu::try_from(signed_note.clone())?;
                        let mut profiles = self.commerce_registries.lock().await;
                        let entry = profiles.entry(signed_note.pubkey.to_string()).or_insert(
                            CommerceRegistryEntry {
                                profile: None,
                                menu: None,
                                whitelisted: false,
                            },
                        );
                        entry.set_menu(signed_note.clone());
                        info!("Added menu");
                    }
                    NOSTR_KIND_COURIER_PROFILE => {
                        let _ = DriverProfile::try_from(signed_note.clone())?;
                        let mut profiles = self.courier_profiles.lock().await;
                        profiles.insert(signed_note.pubkey.to_string(), signed_note);
                        info!("Added courier profile");
                    }
                    NOSTR_KIND_SERVER_CONFIG => {
                        if let Err(e) = self.update_admin_config(signed_note.clone()).await {
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
                        NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                            info!("ORDER REQUEST RECEIVED");
                            if let Err(e) = self.handle_order_request(signed_note).await {
                                error!("ORDER REQUEST ERROR {:?}", e);
                            }
                        }
                        NOSTR_KIND_ORDER_STATE => {
                            if let Err(e) = self.handle_order_state_update(signed_note).await {
                                error!("ORDER STATE ERROR {:?}", e);
                            }
                        }
                        NOSTR_KIND_ADMIN_REQUEST => {
                            if let Ok(admin_req) = AdminServerRequest::try_from(&signed_note) {
                                let mut admin_confs = self.admin_config.lock().await;
                                match admin_req.config_type {
                                    AdminConfigurationType::ExchangeRate => {
                                        admin_confs
                                            .set_exchange_rate(admin_req.config_str.parse()?);
                                        let update = admin_confs.sign_exchange_rate(&self.keys)?;
                                        self.broadcaster.broadcast_note(update).await?;
                                        info!("Exchange rate updated {}", admin_req.config_str);
                                    }
                                    AdminConfigurationType::CommerceWhitelist => {
                                        let whitelist: Vec<String> =
                                            serde_json::from_str(&admin_req.config_str)?;
                                        admin_confs.set_commerce_whitelist(whitelist);
                                        let update =
                                            admin_confs.sign_commerce_whitelist(&self.keys)?;
                                        info!("New commerce whitelist: {}", &update);
                                        self.broadcaster.broadcast_note(update).await?;
                                        info!("Commerce whitelist updated");
                                    }
                                    AdminConfigurationType::CourierWhitelist => {
                                        let whitelist: Vec<String> =
                                            serde_json::from_str(&admin_req.config_str)?;
                                        info!("Courier whitelist updated to: {:?}", &whitelist);
                                        admin_confs.set_couriers_whitelist(whitelist);
                                        let update =
                                            admin_confs.sign_couriers_whitelist(&self.keys)?;
                                        self.broadcaster.broadcast_note(update).await?;
                                        info!("Courier whitelist updated");
                                    }
                                    _ => {}
                                }
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
    async fn create_order_invoice(
        &self,
        order: &OrderRequest,
    ) -> anyhow::Result<(LnAddressPaymentRequest, LndHodlInvoice)> {
        let commerce_profiles = self.commerce_registries.lock().await;
        let commerce = commerce_profiles
            .get(&order.commerce)
            .ok_or(anyhow!("Commerce not found"))?;
        let invoice_total_srd = order.products.total();
        let exchange_rate = self.admin_config.lock().await.get_exchange_rate();
        let invoice_amount = ((invoice_total_srd / exchange_rate) * SATOSHIS_IN_ONE_BTC) as u64;
        let invoice = match commerce
            .ln_address()?
            .get_invoice(
                &self.ln_address_client,
                invoice_amount * MILISATOSHIS_IN_ONE_SATOSHI,
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
        let hodl_amount = invoice_amount + ILLUMINODES_FEES + FUENTE_FEES;
        let hodl_invoice = self
            .lightning_wallet
            .get_hodl_invoice(invoice.r_hash()?, hodl_amount)
            .await?;
        info!("Hodl invoice created");
        Ok((invoice, hodl_invoice))
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
                        let mut live_orders = self.live_orders.lock().await;
                        if !live_orders.contains_key(&order_state.id()) {
                            info!(
                                "Order added to live orders {:?}",
                                order_state.get_order_status()
                            );
                            live_orders.insert(order_state.id(), order_state);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Order state channel closed"))
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
    async fn handle_order_request(&self, signed_note: NostrNote) -> anyhow::Result<()> {
        match OrderRequest::try_from(signed_note.clone()) {
            Ok(order) => {
                info!("Order request received");
                let invoice = self.create_order_invoice(&order).await?;
                info!("Order invoice created");
                let state_update = OrderInvoiceState::new(
                    signed_note.clone(),
                    Some(invoice.1),
                    Some(invoice.0.clone()),
                );
                info!("Order state created");
                let state_update_note = state_update.sign_customer_update(&self.keys)?;
                info!("Order state update signed");
                self.broadcaster.broadcast_note(state_update_note).await?;
                let self_clone = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = self_clone.order_payment_notifier(state_update).await {
                        error!("NOTIFIER ERROR {:?}", e);
                    }
                });
                Ok(())
            }
            Err(e) => {
                error!("{:?}", e);
                Err(e)
            }
        }
    }
    async fn settle_htlc(&self, invoice: LnAddressPaymentRequest) -> anyhow::Result<()> {
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
                let _ = self.settle_htlc(invoice).await?;
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
                self.live_orders.lock().await.insert(order.id(), order);
            }
            OrderStatus::Canceled => {
                if let Some(invoice) = order.get_commerce_invoice() {
                    if let Err(e) = self
                        .lightning_wallet
                        .cancel_htlc(invoice.r_hash_url_safe()?)
                        .await
                    {
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
    async fn handle_courier_updates(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<()> {
        let updated_order: OrderInvoiceState =
            OrderInvoiceState::try_from(inner_note.content.to_string())?;
        // Check if order is part of live orders
        let mut orders = self.live_orders.clone().lock_owned().await;
        let live_order = orders
            .get_mut(&updated_order.id())
            .ok_or(anyhow!("Order not found"))?;
        // Check if the courier is already assigned
        if live_order.get_courier().is_none() {
            let new_courier = updated_order
                .get_courier()
                .ok_or(anyhow!("No courier found"))?;
            live_order.update_courier(new_courier);
            self.broadcast_order_update(live_order).await?;
        }
        // Check if the update s coing from asigned courier
        if outer_note.pubkey == live_order.get_courier().unwrap().pubkey {
            self.broadcast_order_update(&updated_order).await?;
            info!("Order updated by courier");
        }
        Err(anyhow!("Invalid courier update"))
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
                self.lightning_wallet
                    .cancel_htlc(invoice.r_hash_url_safe()?)
                    .await?;
                warn!("Canceled HTLC due to inactivity");
                self.update_order_status(
                    &mut order_state,
                    OrderPaymentStatus::PaymentFailed,
                    OrderStatus::Canceled,
                )
                .await?;
            }
        }
        if let Ok(_) = self
            .admin_config
            .clone()
            .lock_owned()
            .await
            .check_couriers_whitelist(&signed_note.pubkey)
        {
            self.handle_courier_updates(signed_note.clone(), signed_note.clone())
                .await?;
        }
        info!("Order state updated");
        Ok(())
    }
    async fn update_admin_config(&self, signed_note: NostrNote) -> anyhow::Result<()> {
        self.admin_config
            .lock()
            .await
            .check_admin_whitelist(&signed_note.pubkey)?;
        let config_type: AdminConfigurationType = signed_note
            .tags
            .find_tags(NostrTag::Parameterized)
            .get(2)
            .ok_or(anyhow!("No config type found"))?
            .clone()
            .try_into()?;
        match config_type {
            AdminConfigurationType::CommerceWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&signed_note.content)?;
                info!("Commerce whitelist update {:?}", whitelist);
                let mut configs = self.admin_config.lock().await;
                let mut registered_commerces = self.commerce_registries.lock().await;
                for (key, entry) in registered_commerces.iter_mut() {
                    if whitelist.contains(&key) {
                        entry.set_whitelisted(true);
                    } else {
                        entry.set_whitelisted(false);
                    }
                }
                configs.set_commerce_whitelist(whitelist);
                info!("Commerce whitelist updated");
            }
            AdminConfigurationType::CourierWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&signed_note.content)?;
                self.admin_config
                    .lock()
                    .await
                    .set_couriers_whitelist(whitelist);
            }
            AdminConfigurationType::ConsumerBlacklist => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let blacklist: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_consumer_blacklist(blacklist);
            }
            AdminConfigurationType::UserRegistrations => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let registrations: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_user_registrations(registrations);
            }
            AdminConfigurationType::ExchangeRate => {
                let rate: f64 = serde_json::from_str(&signed_note.content)?;
                let mut configs = self.admin_config.lock().await;
                configs.set_exchange_rate(rate);
                info!("Exchange rate set to: {}", rate);
            }
            AdminConfigurationType::AdminWhitelist => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let whitelist: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_admin_whitelist(whitelist);
            }
        }
        Ok(())
    }
}
