const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];
use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
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
use lightning::{
    HodlState, InvoicePaymentState, LightningClient, LnAddressPaymentRequest, LndHodlInvoice,
    LndPaymentRequest,
};
use nostro2::{notes::SignedNote, pool::RelayPool, relays::NostrFilter, userkeys::UserKeys};
use tokio::sync::Mutex;
use tracing::{error, info, warn, Level};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommerceRegistryEntry {
    profile: Option<SignedNote>,
    menu: Option<SignedNote>,
    whitelisted: bool,
}
impl CommerceRegistryEntry {
    pub fn new(profile: Option<SignedNote>, menu: Option<SignedNote>) -> Self {
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
    pub fn get_profile(&self) -> Option<SignedNote> {
        self.profile.clone()
    }
    pub fn get_menu(&self) -> Option<SignedNote> {
        self.menu.clone()
    }
    pub fn set_profile(&mut self, profile: SignedNote) {
        self.profile = Some(profile);
    }
    pub fn set_menu(&mut self, menu: SignedNote) {
        self.menu = Some(menu);
    }
    pub fn ln_address(&self) -> anyhow::Result<String> {
        let profile = self.profile.clone().ok_or(anyhow!("No profile found"))?;
        let commerce_profile = CommerceProfile::try_from(profile)?;
        Ok(commerce_profile.ln_address().to_string())
    }
}

#[derive(Clone)]
pub struct InvoicerBot {
    keys: UserKeys,
    relays: RelayPool,
    admin_config: Arc<Mutex<AdminConfiguration>>,
    courier_profiles: Arc<Mutex<HashMap<String, SignedNote>>>,
    commerce_registries: Arc<Mutex<HashMap<String, CommerceRegistryEntry>>>,
    live_orders: Arc<Mutex<HashMap<String, OrderInvoiceState>>>,
    lightning_wallet: LightningClient,
}

impl InvoicerBot {
    pub async fn new() -> anyhow::Result<Self> {
        let lightning_wallet =
            LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let urls = RELAY_URLS.iter().map(|s| s.to_string()).collect();
        let relays = RelayPool::new(urls).await?;
        let server_keys = UserKeys::new(TEST_PRIV_KEY)?;
        let mut admin_config = AdminConfiguration::default();
        admin_config.set_admin_whitelist(vec![TEST_PUB_KEY.to_string()]);
        Ok(Self {
            keys: server_keys,
            relays,
            admin_config: Arc::new(Mutex::new(admin_config)),
            courier_profiles: Arc::new(Mutex::new(HashMap::new())),
            commerce_registries: Arc::new(Mutex::new(HashMap::new())),
            live_orders: Arc::new(Mutex::new(HashMap::new())),
            lightning_wallet,
        })
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
        let invoice_amount = ((invoice_total_srd / exchange_rate) * 100_000_000.0) as u64;
        let invoice = self
            .lightning_wallet
            .get_ln_url_invoice(invoice_amount * 1000, commerce.ln_address()?)
            .await?;
        // TODO
        // Add Mayas profit to the invoice
        let hodl_amount = invoice_amount + 200;
        let hodl_invoice = self
            .lightning_wallet
            .get_hodl_invoice(invoice.r_hash()?, hodl_amount)
            .await?;
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
    async fn order_payment_notifier(self, mut state: OrderInvoiceState) -> anyhow::Result<bool> {
        let invoice = state
            .get_commerce_invoice()
            .ok_or(anyhow!("No commerce invoice found"))?;
        let subscriber = self
            .lightning_wallet
            .subscribe_to_invoice(invoice.r_hash_url_safe()?)
            .await?;
        let mut success = false;
        loop {
            if subscriber.is_closed() {
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
            if let Ok(status) = subscriber.recv().await {
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
    async fn handle_order_request(self, signed_note: SignedNote) -> anyhow::Result<()> {
        if let Ok(order) = OrderRequest::try_from(signed_note.clone()) {
            let invoice = self.create_order_invoice(&order).await?;
            let state_update = OrderInvoiceState::new(
                signed_note.clone(),
                Some(invoice.1),
                Some(invoice.0.clone()),
            );
            let state_update_note = state_update.sign_customer_update(&self.keys)?;
            self.relays.broadcast_note(state_update_note).await?;
            let self_clone = self.clone();
            tokio::spawn(async move {
                if let Err(e) = self_clone.order_payment_notifier(state_update).await {
                    error!("{:?}", e);
                }
            });
        }

        Ok(())
    }
    async fn settle_htlc(&self, invoice: LnAddressPaymentRequest) -> anyhow::Result<()> {
        let payment_req = LndPaymentRequest::new(invoice.pr, 10, 150.to_string(), false);
        let (payment_rx, payment_tx) = self.lightning_wallet.invoice_channel().await?;
        payment_tx.send(payment_req).await?;
        loop {
            if payment_rx.is_closed() && payment_rx.is_empty() {
                return Err(anyhow!("Payment channel closed"));
            }
            if let Ok(payment_status) = payment_rx.recv().await {
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
        self.relays.broadcast_note(user_note).await?;
        self.relays.broadcast_note(commerce_note).await?;
        match state.get_courier() {
            Some(_) => {
                let courier_note = state.sign_courier_update(&self.keys)?;
                self.relays.broadcast_note(courier_note).await?;
            }
            None => {
                if state.get_order_status() == OrderStatus::ReadyForDelivery {
                    let courier_note = state.sign_courier_update(&self.keys)?;
                    self.relays.broadcast_note(courier_note).await?;
                }
            }
        }
        Ok(())
    }
    async fn handle_commerce_updates(
        &self,
        inner_note: SignedNote,
        outer_note: SignedNote,
    ) -> anyhow::Result<()> {
        let mut order: OrderInvoiceState =
            OrderInvoiceState::try_from(inner_note.get_content().to_string())?;
        if outer_note.get_pubkey() != order.get_order_request().commerce {
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
    async fn handle_courier_profile(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        let _ = DriverProfile::try_from(signed_note.clone())?;
        let mut profiles = self.courier_profiles.lock().await;
        profiles.insert(signed_note.get_pubkey().to_string(), signed_note);
        info!("Added courier profile");
        Ok(())
    }
    async fn handle_commerce_profile(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        CommerceProfile::try_from(signed_note.clone())?;
        let mut profiles = self.commerce_registries.lock().await;
        let entry = profiles
            .entry(signed_note.get_pubkey().to_string())
            .or_insert(CommerceRegistryEntry {
                profile: None,
                menu: None,
                whitelisted: false,
            });
        entry.set_profile(signed_note.clone());
        info!("Added commerce profile");
        Ok(())
    }
    async fn handle_commerce_product_list(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        ProductMenu::try_from(signed_note.clone())?;
        let mut profiles = self.commerce_registries.lock().await;
        let entry = profiles
            .entry(signed_note.get_pubkey().to_string())
            .or_insert(CommerceRegistryEntry {
                profile: None,
                menu: None,
                whitelisted: false,
            });
        entry.set_menu(signed_note.clone());
        info!("Added menu");
        Ok(())
    }
    async fn handle_courier_updates(
        &self,
        inner_note: SignedNote,
        outer_note: SignedNote,
    ) -> anyhow::Result<()> {
        let updated_order: OrderInvoiceState =
            OrderInvoiceState::try_from(inner_note.get_content().to_string())?;
        // Check if order is part of live orders
        let mut orders = self.live_orders.lock().await;
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
        if outer_note.get_pubkey() == live_order.get_courier().unwrap().get_pubkey() {
            self.broadcast_order_update(&updated_order).await?;
            info!("Order updated by courier");
        }
        Ok(())
    }
    async fn handle_order_state_update(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        let mut order_state = OrderInvoiceState::try_from(signed_note.get_content())?;
        let commerce = order_state.get_order_request().commerce.clone();
        if signed_note.get_pubkey() == commerce {
            if let Err(e) = self
                .clone()
                .handle_commerce_updates(signed_note.clone(), signed_note.clone())
                .await
            {
                error!("{:?}", e);
            }
        }
        if signed_note.get_pubkey() == order_state.get_order().get_pubkey() {
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
        match self
            .admin_config
            .lock()
            .await
            .check_couriers_whitelist(&signed_note.get_pubkey())
        {
            Ok(_) => {
                self.handle_courier_updates(signed_note.clone(), signed_note.clone())
                    .await?;
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        info!("Order state updated");
        Ok(())
    }
    async fn update_admin_config(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        self.admin_config
            .lock()
            .await
            .check_admin_whitelist(&signed_note.get_pubkey())?;
        let config_type: AdminConfigurationType = signed_note
            .get_tags_by_id("d")
            .ok_or(anyhow!("No config type found"))?
            .get(2)
            .ok_or(anyhow!("No config type found"))?
            .clone()
            .try_into()?;
        match config_type {
            AdminConfigurationType::CommerceWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&signed_note.get_content())?;
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
                let whitelist: Vec<String> = serde_json::from_str(&signed_note.get_content())?;
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
                let rate: f64 = serde_json::from_str(&signed_note.get_content())?;
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
    pub async fn note_processor(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        match signed_note.get_kind() {
            NOSTR_KIND_SERVER_REQUEST => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = SignedNote::try_from(decrypted)?;
                match inner_note.get_kind() {
                    NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                        self.clone().handle_order_request(inner_note).await?;
                    }
                    NOSTR_KIND_ORDER_STATE => {
                        self.handle_order_state_update(inner_note).await?;
                    }
                    _ => {
                        return Err(anyhow!("Invalid inner note"));
                    }
                }
            }
            NOSTR_KIND_ADMIN_REQUEST => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = SignedNote::try_from(decrypted)?;
                let admin_req = AdminServerRequest::try_from(&inner_note)?;
                let mut admin_confs = self.admin_config.lock().await;
                match admin_req.config_type {
                    AdminConfigurationType::ExchangeRate => {
                        admin_confs.set_exchange_rate(admin_req.config_str.parse()?);
                        let update = admin_confs.sign_exchange_rate(&self.keys)?;
                        self.relays.broadcast_note(update).await?;
                        info!("Exchange rate updated {}", admin_req.config_str);
                    }
                    AdminConfigurationType::CommerceWhitelist => {
                        let whitelist: Vec<String> = serde_json::from_str(&admin_req.config_str)?;
                        admin_confs.set_commerce_whitelist(whitelist);
                        let update = admin_confs.sign_commerce_whitelist(&self.keys)?;
                        info!("New commerce whitelist: {}", &update);
                        self.relays.broadcast_note(update).await?;
                        info!("Commerce whitelist updated");
                    }
                    AdminConfigurationType::CourierWhitelist => {
                        let whitelist: Vec<String> = serde_json::from_str(&admin_req.config_str)?;
                        info!("Courier whitelist updated to: {:?}", &whitelist);
                        admin_confs.set_couriers_whitelist(whitelist);
                        let update = admin_confs.sign_couriers_whitelist(&self.keys)?;
                        self.relays.broadcast_note(update).await?;
                        info!("Courier whitelist updated");
                    }
                    _ => {}
                }
            }

            NOSTR_KIND_SERVER_CONFIG => {
                info!("Server config received");
                if let Err(e) = self.update_admin_config(signed_note).await {
                    error!("{:?}", e);
                }
            }
            NOSTR_KIND_ORDER_STATE => {
                let courier_hub_keys = UserKeys::new(DRIVER_HUB_PRIV_KEY)?;
                let decrypted = courier_hub_keys.decrypt_nip_04_content(&signed_note)?;
                let order_state = OrderInvoiceState::try_from(decrypted)?;
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
                            info!("Order added to live orders {:?}", order_state.get_order_status());
                            live_orders.insert(order_state.id(), order_state);
                        }
                    }
                }
            }
            NOSTR_KIND_COMMERCE_PROFILE => {
                self.handle_commerce_profile(signed_note).await?;
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                self.handle_commerce_product_list(signed_note).await?;
            }
            NOSTR_KIND_COURIER_PROFILE => {
                self.admin_config
                    .lock()
                    .await
                    .check_couriers_whitelist(&signed_note.get_pubkey())?;
                self.handle_courier_profile(signed_note).await?;
            }
            _ => {
                return Err(anyhow!("Invalid note kind"));
            }
        }
        Ok(())
    }
    pub async fn read_relay_pool(self) -> anyhow::Result<()> {
        let live_filter = NostrFilter::default()
            .new_kinds(vec![NOSTR_KIND_ORDER_STATE])
            .new_tag("p", vec![DRIVER_HUB_PUB_KEY.to_string()]);
        let filter = NostrFilter::default()
            .new_kinds(vec![
                NOSTR_KIND_SERVER_REQUEST,
                NOSTR_KIND_SERVER_CONFIG,
                NOSTR_KIND_ADMIN_REQUEST,
            ])
            .new_tag("p", vec![TEST_PUB_KEY.to_string()]);
        let commerces_filter = NostrFilter::default().new_kinds(vec![
            NOSTR_KIND_COMMERCE_PROFILE,
            NOSTR_KIND_COMMERCE_PRODUCTS,
            NOSTR_KIND_COURIER_PROFILE,
        ]);
        let config_filter = NostrFilter::default()
            .new_kind(NOSTR_KIND_SERVER_CONFIG)
            .new_author(TEST_PUB_KEY);
        self.relays.subscribe(filter.subscribe()).await?;
        self.relays.subscribe(commerces_filter.subscribe()).await?;
        self.relays.subscribe(live_filter.subscribe()).await?;
        self.relays.subscribe(config_filter.subscribe()).await?;
        let reader = self.relays.pooled_notes();
        // let events = self.relays.all_events();
        // tokio::spawn(async move {
        //     while let Ok(event) = events.recv().await {
        //         match event {
        //             nostro2::relays::RelayEvents::EVENT(_, _) => {}
        //             _ => {
        //                 info!("{:?}", event);
        //             }
        //         }
        //     }
        // });
        loop {
            if reader.is_closed() {
                break;
            }
            if let Ok(signed_note) = reader.recv().await {
                if let Err(e) = self.clone().note_processor(signed_note).await {
                    error!("{:?}", e);
                }
            }
        }
        reader.close();
        Err(anyhow!("Relay pool closed"))
    }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();
    info!("Bot created");
    let mut close_counters = 0;
    loop {
        if close_counters > 10 {
            break;
        }
        info!("Starting Invoicer");
        if close_counters > 0 {
            info!("Reconnecting for the {} time", close_counters);
        }
        let bot = InvoicerBot::new().await?;
        if let Err(e) = bot.clone().read_relay_pool().await {
            error!("{:?}", e);
            close_counters += 1;
        }
    }
    Ok(())
}
