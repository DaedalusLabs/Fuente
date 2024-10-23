const RELAY_URLS: [&str; 2] = ["wss://relay.illuminodes.com", "wss://relay.arrakis.lat"];
use std::{collections::HashMap, sync::Arc};
mod courier_bot;
pub mod lnd;

use anyhow::anyhow;
use fuente::models::{
    commerce::CommerceProfile,
    lnd::{HodlState, InvoicePaymentState, LndHodlInvoice, LndInvoice, LndPaymentRequest},
    nostr_kinds::{
        NOSTR_KIND_COMMERCE_PRODUCTS, NOSTR_KIND_COMMERCE_PROFILE,
        NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_ORDER_STATE, NOSTR_KIND_SERVER_REQUEST,
    },
    orders::{OrderInvoiceState, OrderPaymentStatus, OrderRequest, OrderStatus},
    products::{ProductMenu, ProductOrder},
    TEST_PRIV_KEY, TEST_PUB_KEY,
};
use lnd::LndClient;
use nostro2::{notes::SignedNote, pool::RelayPool, relays::NostrFilter, userkeys::UserKeys};
use tokio::sync::Mutex;
use tracing::{error, info, warn, Level};

#[derive(Clone)]
pub struct InvoicerBot {
    keys: UserKeys,
    relays: RelayPool,
    commerce_whitelist: Arc<Mutex<Vec<String>>>,
    courier_whitelist: Arc<Mutex<Vec<String>>>,
    courier_profiles: Arc<Mutex<HashMap<String, SignedNote>>>,
    commerce_profiles: Arc<Mutex<HashMap<String, CommerceProfile>>>,
    menu_notes: Arc<Mutex<HashMap<String, ProductMenu>>>,
    server_wallet: LndClient,
    commerce_wallets: LndClient,
}
impl InvoicerBot {
    pub async fn new() -> anyhow::Result<Self> {
        let server_wallet = LndClient::dud_server().await?;
        // let server_wallet = LndClient::new("localhost:2100", "../test/lnddatadir/").await?;
        // TODO
        // We wont need this second client cause we
        // will get commerce invoices from their LNURLs
        // let commerce_wallets =
        //     LndClient::new("localhost:4201", "../test/commerce_lnd_test/").await?;
        let commerce_wallets = LndClient::dud_server().await?;
        let urls = RELAY_URLS.iter().map(|s| s.to_string()).collect();
        let relays = RelayPool::new(urls).await?;
        Ok(Self {
            keys: UserKeys::new(TEST_PRIV_KEY).unwrap(),
            relays,
            commerce_whitelist: Arc::new(Mutex::new(Vec::new())),
            courier_whitelist: Arc::new(Mutex::new(vec![
                "4d8b9d6f52bde35923dcd1eef872be8f5c94f8374f6bd46f3ad54b85112afe26".to_string(),
            ])),
            courier_profiles: Arc::new(Mutex::new(HashMap::new())),
            commerce_profiles: Arc::new(Mutex::new(HashMap::new())),
            menu_notes: Arc::new(Mutex::new(HashMap::new())),
            server_wallet,
            commerce_wallets,
        })
    }
    async fn create_order_invoice(
        &self,
        order: ProductOrder,
    ) -> anyhow::Result<(LndInvoice, LndHodlInvoice)> {
        let invoice_amount = order.total() as u64 * 100;
        let invoice = self.commerce_wallets.get_invoice(invoice_amount).await?;
        // We create a invoice for user to pay
        // Value i the amount of the order + 200 illuminodes fee + whatever profit Maya
        // wants to make
        let hodl_amount = invoice_amount + 200 + 1000;
        let hodl_invoice = self
            .server_wallet
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
        let state_note = state.sign_customer_update(&self.keys)?;
        self.relays.broadcast_note(state_note).await?;
        let commerce_state_note = state.sign_commerce_update(&self.keys)?;
        self.relays.broadcast_note(commerce_state_note).await?;
        Ok(())
    }
    async fn order_payment_notifier(self, mut state: OrderInvoiceState) -> anyhow::Result<bool> {
        let invoice = state
            .get_commerce_invoice()
            .ok_or(anyhow!("No commerce invoice found"))?;
        let subscriber = self
            .server_wallet
            .subscribe_to_invoice(invoice.clone())
            .await?;
        let mut success = false;
        loop {
            if subscriber.is_closed() {
                self.server_wallet
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
                        self.server_wallet
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
        let (payment_rx, payment_tx) = self.server_wallet.invoice_channel().await?;
        payment_tx.send(payment_req).await?;
        while let Ok(payment_status) = payment_rx.recv().await {
            if payment_status.status() == InvoicePaymentState::Succeeded {
                let _ = self
                    .server_wallet
                    .settle_htlc(payment_status.preimage())
                    .await;
                break;
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
                let driver_note = order.sign_driver_update(&self.keys)?;
                let user_note = order.sign_customer_update(&self.keys)?;
                let commerce_note = order.sign_commerce_update(&self.keys)?;
                self.relays.broadcast_note(driver_note).await?;
                self.relays.broadcast_note(user_note).await?;
                self.relays.broadcast_note(commerce_note).await?;
            }
            OrderStatus::Canceled => {
                if let Some(invoice) = order.get_commerce_invoice() {
                    if let Err(e) = self
                        .server_wallet
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
    pub async fn note_processor(&self, signed_note: SignedNote) -> anyhow::Result<()> {
        match signed_note.get_kind() {
            NOSTR_KIND_SERVER_REQUEST => {
                let decrypted = self.keys.decrypt_nip_04_content(&signed_note)?;
                let inner_note = SignedNote::try_from(decrypted)?;
                match inner_note.get_kind() {
                    NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
                        self.clone()
                            .handle_order_request(inner_note.clone())
                            .await?;
                    }
                    NOSTR_KIND_ORDER_STATE => {
                        let mut order_state = OrderInvoiceState::try_from(inner_note.get_content())?;
                        let commerce = order_state.get_order_request().commerce.clone();
                        if signed_note.get_pubkey() == commerce {
                            if let Err(e) = self
                                .clone()
                                .handle_commerce_updates(inner_note.clone(), signed_note.clone())
                                .await
                            {
                                error!("{:?}", e);
                            }
                        }
                        let couriers = self.courier_whitelist.lock().await;
                        if couriers.contains(&signed_note.get_pubkey().to_string()) {
                            // order_state.update_courier_status(order_state.clone());
                        }
                    }
                    _ => {}
                }
            }
            NOSTR_KIND_COMMERCE_PROFILE => {
                let profile = CommerceProfile::try_from(signed_note.clone())?;
                let mut profiles = self.commerce_profiles.lock().await;
                profiles.insert(signed_note.get_pubkey().to_string(), profile.clone());
                info!("Added commerce profile");
            }
            NOSTR_KIND_COMMERCE_PRODUCTS => {
                let menu = ProductMenu::try_from(signed_note.clone())?;
                let mut menus = self.menu_notes.lock().await;
                menus.insert(signed_note.get_pubkey().to_string(), menu.clone());
                info!("Added menu");
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn read_relay_pool(self) -> anyhow::Result<()> {
        let filter = NostrFilter::default()
            .new_kind(NOSTR_KIND_SERVER_REQUEST)
            .new_tag("p", vec![TEST_PUB_KEY.to_string()]);
        let commerces_filter = NostrFilter::default().new_kinds(vec![
            NOSTR_KIND_COMMERCE_PROFILE,
            NOSTR_KIND_COMMERCE_PRODUCTS,
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
        // while let Ok(signed_note) = reader.recv().await {
        //     match signed_note.get_kind() {
        //         NOSTR_KIND_SERVER_REQUEST => {
        //             if let Ok(decrypted) = self.keys.decrypt_nip_04_content(&signed_note) {
        //                 if let Ok(inner_note) = SignedNote::try_from(decrypted) {
        //                     match inner_note.get_kind() {
        //                         NOSTR_KIND_CONSUMER_ORDER_REQUEST => {
        //                             if let Err(e) =
        //                                 self.clone().handle_order_request(inner_note.clone()).await
        //                             {
        //                                 error!("{:?}", e);
        //                             }
        //                         }
        //                         NOSTR_KIND_ORDER_STATE => {
        //                             if let Ok(order_state) =
        //                                 OrderInvoiceState::try_from(inner_note.clone())
        //                             {
        //                                 let commerce =
        //                                     order_state.get_order_request().commerce.clone();
        //                                 if signed_note.get_pubkey() == commerce {
        //                                     if let Err(e) = self
        //                                         .clone()
        //                                         .handle_commerce_updates(
        //                                             inner_note.clone(),
        //                                             signed_note.clone(),
        //                                         )
        //                                         .await
        //                                     {
        //                                         error!("{:?}", e);
        //                                     }
        //                                 }
        //                                 let couriers = self.courier_whitelist.lock().await;
        //                                 if couriers.contains(&signed_note.get_pubkey().to_string())
        //                                 {
        //                                     info!("Courier update");
        //                                 }
        //                             }
        //                         }
        //                         _ => {}
        //                     }
        //                 }
        //             }
        //         }
        //         NOSTR_KIND_COMMERCE_PROFILE => {
        //             if let Ok(profile) = CommerceProfile::try_from(signed_note.clone()) {
        //                 let mut profiles = self.commerce_profiles.lock().await;
        //                 profiles.insert(signed_note.get_pubkey().to_string(), profile.clone());
        //                 info!("Added commerce profile");
        //             }
        //         }
        //         NOSTR_KIND_COMMERCE_PRODUCTS => {
        //             if let Ok(menu) = ProductMenu::try_from(signed_note.clone()) {
        //                 let mut menus = self.menu_notes.lock().await;
        //                 menus.insert(signed_note.get_pubkey().to_string(), menu.clone());
        //                 info!("Added menu");
        //             };
        //         }
        //         _ => {}
        //     }
        // }
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
