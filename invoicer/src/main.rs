const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];
use std::{collections::HashMap, sync::Arc};
pub mod lnd;

use anyhow::anyhow;
use fuente::models::{
    admin_configs::{AdminConfiguration, AdminConfigurationType, AdminServerRequest},
    commerce::CommerceProfile,
    driver::DriverProfile,
    lnd::{HodlState, InvoicePaymentState, LndHodlInvoice, LndInvoice, LndPaymentRequest},
    nostr_kinds::{
        NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
        NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_ORDER_STATE,
        NOSTR_KIND_SERVER_CONFIG, NOSTR_KIND_SERVER_REQUEST,
    },
    orders::{OrderInvoiceState, OrderPaymentStatus, OrderRequest, OrderStatus},
    products::{ProductMenu, ProductOrder},
    TEST_PRIV_KEY, TEST_PUB_KEY,
};
use lnd::LightningClient;
use nostro2::{notes::SignedNote, pool::RelayPool, relays::NostrFilter, userkeys::UserKeys};
use tokio::sync::Mutex;
use tracing::{error, info, warn, Level};

#[derive(Clone)]
pub struct InvoicerBot {
    keys: UserKeys,
    relays: RelayPool,
    admin_config: Arc<Mutex<AdminConfiguration>>,
    courier_profiles: Arc<Mutex<HashMap<String, SignedNote>>>,
    commerce_profiles: Arc<Mutex<HashMap<String, CommerceProfile>>>,
    menu_notes: Arc<Mutex<HashMap<String, ProductMenu>>>,
    live_orders: Arc<Mutex<HashMap<String, OrderInvoiceState>>>,
    lightning_wallet: LightningClient,
}
impl InvoicerBot {
    pub async fn new() -> anyhow::Result<Self> {
        let lightning_wallet = LightningClient::dud_server().await?;
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
            commerce_profiles: Arc::new(Mutex::new(HashMap::new())),
            menu_notes: Arc::new(Mutex::new(HashMap::new())),
            live_orders: Arc::new(Mutex::new(HashMap::new())),
            lightning_wallet,
        })
    }
    async fn create_order_invoice(
        &self,
        order: ProductOrder,
    ) -> anyhow::Result<(LndInvoice, LndHodlInvoice)> {
        let invoice_amount = order.total() as u64 * 100;
        let invoice = self.lightning_wallet.get_invoice(invoice_amount).await?;
        // We create a invoice for user to pay
        // Value i the amount of the order + 200 illuminodes fee + whatever profit Maya
        // wants to make
        let hodl_amount = invoice_amount + 200 + 1000;
        let hodl_invoice = self
            .lightning_wallet
            .get_hodl_invoice(invoice.r_hash(), hodl_amount)
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
            .subscribe_to_invoice(invoice.r_hash_url_safe())
            .await?;
        let mut success = false;
        loop {
            if subscriber.is_closed() {
                self.lightning_wallet
                    .cancel_htlc(invoice.r_hash_url_safe())
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
                            .cancel_htlc(invoice.r_hash_url_safe())
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
        let order: OrderRequest = signed_note.clone().try_into()?;
        let invoice = self.create_order_invoice(order.products).await?;
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
        Ok(())
    }
    async fn settle_htlc(&self, invoice: LndInvoice) -> anyhow::Result<()> {
        let payment_req =
            LndPaymentRequest::new(invoice.payment_request(), 120, 100.to_string(), false);
        let (payment_rx, payment_tx) = self.lightning_wallet.invoice_channel().await?;
        payment_tx.send(payment_req).await?;
        while let Ok(payment_status) = payment_rx.recv().await {
            if payment_status.status() == InvoicePaymentState::Succeeded {
                let _ = self
                    .lightning_wallet
                    .settle_htlc(payment_status.preimage())
                    .await;
                break;
            }
        }
        Ok(())
    }
    async fn broadcast_order_update(&self, state: &OrderInvoiceState) -> anyhow::Result<()> {
        let user_note = state.sign_customer_update(&self.keys)?;
        let commerce_note = state.sign_commerce_update(&self.keys)?;
        self.relays.broadcast_note(user_note).await?;
        self.relays.broadcast_note(commerce_note).await?;
        if state.get_courier().is_some() {
            let courier_note = state.sign_courier_update(&self.keys)?;
            self.relays.broadcast_note(courier_note).await?;
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
                        .cancel_htlc(invoice.r_hash_url_safe())
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
        let profile = CommerceProfile::try_from(signed_note.clone())?;
        let mut profiles = self.commerce_profiles.lock().await;
        profiles.insert(signed_note.get_pubkey().to_string(), profile.clone());
        info!("Added commerce profile");
        Ok(())
    }
    async fn handle_commerce_product_list(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        let menu = ProductMenu::try_from(signed_note.clone())?;
        let mut menus = self.menu_notes.lock().await;
        menus.insert(signed_note.get_pubkey().to_string(), menu.clone());
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
        }
        Ok(())
    }
    async fn handle_order_state_update(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        let order_state = OrderInvoiceState::try_from(signed_note.get_content())?;
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
        self.admin_config
            .lock()
            .await
            .check_couriers_whitelist(&signed_note.get_pubkey())?;
        self.handle_courier_updates(signed_note.clone(), signed_note.clone())
            .await?;
        Ok(())
    }
    async fn update_admin_config(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        self.admin_config
            .lock()
            .await
            .check_admin_whitelist(&signed_note.get_pubkey())?;
        let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
        let config_type: AdminConfigurationType = signed_note
            .get_tags_by_id("d")
            .ok_or(anyhow!("No config type found"))?
            .get(2)
            .ok_or(anyhow!("No config type found"))?
            .clone()
            .try_into()?;
        match config_type {
            AdminConfigurationType::CommerceWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_commerce_whitelist(whitelist);
            }
            AdminConfigurationType::CourierWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_couriers_whitelist(whitelist);
            }
            AdminConfigurationType::ConsumerBlacklist => {
                let blacklist: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_consumer_blacklist(blacklist);
            }
            AdminConfigurationType::UserRegistrations => {
                let registrations: Vec<String> = serde_json::from_str(&decrypted)?;
                self.admin_config
                    .lock()
                    .await
                    .set_user_registrations(registrations);
            }
            AdminConfigurationType::ExchangeRate => {
                let rate: f64 = serde_json::from_str(&decrypted)?;
                self.admin_config.lock().await.set_exchange_rate(rate);
                info!("Exchange rate set to: {}", rate);
            }
            AdminConfigurationType::AdminWhitelist => {
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
                match admin_req.config_type {
                    AdminConfigurationType::ExchangeRate => {
                        let mut admin_confs = self.admin_config.lock().await;
                        info!("{}", &admin_req.config_str);
                        admin_confs.set_exchange_rate(admin_req.config_str.parse()?);
                        let update = admin_confs
                            .sign_exchange_rate(&self.keys, self.keys.get_public_key())?;
                        self.relays.broadcast_note(update).await?;
                        let admin_update =
                            admin_confs.sign_exchange_rate(&self.keys, signed_note.get_pubkey())?;
                        self.relays.broadcast_note(admin_update).await?;
                    }
                    _ => {}
                }
            }
            NOSTR_KIND_SERVER_CONFIG => {
                self.update_admin_config(signed_note).await?;
            }
            NOSTR_KIND_COMMERCE_PROFILE => {
                self.admin_config
                    .lock()
                    .await
                    .check_commerce_whitelist(&signed_note.get_pubkey())?;
                self.handle_commerce_profile(signed_note).await?;
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                self.admin_config
                    .lock()
                    .await
                    .check_commerce_whitelist(&signed_note.get_pubkey())?;
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
        self.relays.subscribe(filter.subscribe()).await?;
        self.relays.subscribe(commerces_filter.subscribe()).await?;
        let reader = self.relays.pooled_notes();
        // let events = self.relays.all_events();
        // tokio::spawn(async move {
        //     while let Ok(event) = events.recv().await {
        //         if let RelayEvents::OK(_, _, dupe) = event {
        //             if dupe != "" {
        //                 warn!("Duplicate note received");
        //             } else {
        //                 info!("Note sent");
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
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();
    info!("Starting Invoicer");
    let bot = InvoicerBot::new().await?;
    info!("Bot created");
    if let Err(e) = bot.read_relay_pool().await {
        error!("{:?}", e);
    }
    Ok(())
}
