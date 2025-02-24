use std::sync::Arc;

use anyhow::anyhow;
use fuente::models::{
    AdminConfiguration, AdminConfigurationType, AdminServerRequest, CommerceProfile,
    OrderInvoiceState, ProductMenu,
};
use nostro2::{
    keypair::NostrKeypair,
    notes::{NostrNote, NostrTag},
};
use tokio::sync::RwLock;
use tracing::info;

use crate::registries::{
    CommerceRegistry, CommerceRegistryEntry, ConsumerRegistry, ConsumerRegistryEntry,
    CourierRegistry, CourierRegistryEntry, LiveOrders,
};

#[derive(Debug, Clone)]
struct InvoicerState {
    consumer_profiles: ConsumerRegistry,
    courier_profiles: CourierRegistry,
    commerce_registries: CommerceRegistry,
    live_orders: LiveOrders,
    admin_config: AdminConfiguration,
}
impl InvoicerState {
    fn read_whitelist() -> Vec<String> {
        let whitelist = include_str!("whitelist.txt");
        tracing::debug!("Whitelist: {}", whitelist);
        whitelist
            .trim()
            .lines()
            .map(|x| x.trim().to_string())
            .collect()
    }
    pub fn new() -> Self {
        let mut admin_config = AdminConfiguration::default();
        admin_config.set_admin_whitelist(Self::read_whitelist());
        Self {
            consumer_profiles: ConsumerRegistry::default(),
            courier_profiles: CourierRegistry::default(),
            commerce_registries: CommerceRegistry::default(),
            live_orders: LiveOrders::default(),
            admin_config,
        }
    }
}
#[derive(Clone)]
pub struct InvoicerStateLock(Arc<RwLock<InvoicerState>>);
impl Default for InvoicerStateLock {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(InvoicerState::new())))
    }
}
impl InvoicerStateLock {
    async fn lock(&self) -> tokio::sync::RwLockWriteGuard<'_, InvoicerState> {
        self.0.write().await
    }
    async fn lock_owned(&self) -> InvoicerState {
        self.0.read().await.clone()
    }
    pub async fn find_whitelisted_courier(&self, pubkey: &str) -> anyhow::Result<NostrNote> {
        let profiles = self.lock().await;
        profiles.admin_config.check_couriers_whitelist(pubkey)?;
        let courier = profiles
            .courier_profiles
            .find_courier(pubkey)
            .ok_or(anyhow!("Courier not found"))?;
        Ok(courier)
    }
    pub async fn is_commerce_whitelisted(&self, pubkey: &str) -> bool {
        self.lock_owned()
            .await
            .admin_config
            .check_commerce_whitelist(pubkey)
            .is_ok()
    }
    pub async fn is_consumer_registered(&self, pubkey: &str) -> bool {
        let profiles = self.lock_owned().await;
        profiles
            .admin_config
            .check_consumer_blacklist(pubkey)
            .is_ok()
            && profiles.consumer_profiles.is_registered(pubkey)
    }
    pub async fn exchange_rate(&self) -> f64 {
        self.lock().await.admin_config.get_exchange_rate()
    }
    pub async fn find_commerce(
        &self,
        pubkey: &str,
    ) -> anyhow::Result<(CommerceProfile, ProductMenu)> {
        let profiles = self.lock().await;
        profiles.admin_config.check_commerce_whitelist(pubkey)?;
        let commerce_entry = profiles
            .commerce_registries
            .get_commerce(pubkey)
            .ok_or(anyhow!("Commerce not found"))?;
        let profile = commerce_entry
            .profile
            .as_ref()
            .ok_or(anyhow!("No profile found"))?;
        let menu = commerce_entry
            .menu
            .as_ref()
            .ok_or(anyhow!("No menu found"))?;
        let commerce_profile = CommerceProfile::try_from(profile.clone())?;
        let product_menu = ProductMenu::try_from(menu.clone())?;
        Ok((commerce_profile, product_menu))
    }
    pub async fn add_consumer_profile(&self, profile: NostrNote) -> anyhow::Result<()> {
        let mut profiles = self.lock().await;
        profiles.consumer_profiles.insert_consumer(
            profile.pubkey.clone(),
            ConsumerRegistryEntry {
                profile,
                ..Default::default()
            },
        );
        Ok(())
    }
    pub async fn add_commerce_profile(&self, profile: NostrNote) -> anyhow::Result<()> {
        let mut profiles = self.lock().await;
        profiles.commerce_registries.update_record(
            profile.pubkey.clone(),
            CommerceRegistryEntry {
                profile: Some(profile),
                ..Default::default()
            },
        );
        Ok(())
    }
    pub async fn add_commerce_menu(&self, menu: NostrNote) -> anyhow::Result<()> {
        let mut profiles = self.lock().await;
        profiles.commerce_registries.update_record(
            menu.pubkey.clone(),
            CommerceRegistryEntry {
                menu: Some(menu),
                ..Default::default()
            },
        );
        Ok(())
    }
    pub async fn add_courier_profile(&self, profile: NostrNote) -> anyhow::Result<()> {
        let mut profiles = self.lock().await;
        profiles.courier_profiles.insert_courier(
            profile.pubkey.clone(),
            CourierRegistryEntry {
                profile,
                ..Default::default()
            },
        );
        Ok(())
    }
    pub async fn update_live_order(&self, order: NostrNote) -> anyhow::Result<()> {
        let mut orders = self.lock().await;
        let invoice_state = OrderInvoiceState::try_from(order)?;
        orders
            .live_orders
            .update_order_record(invoice_state.order_id(), invoice_state)?;
        Ok(())
    }
    pub async fn remove_live_order(&self, order_id: &str) -> anyhow::Result<()> {
        self.lock().await.live_orders.remove_order(order_id)
    }
    pub async fn find_live_order(&self, order_id: &str) -> Option<OrderInvoiceState> {
        self.lock_owned().await.live_orders.get_order(order_id)
    }
    pub async fn sign_updated_config(
        &self,
        admin_note: NostrNote,
        signing_keys: &NostrKeypair,
    ) -> anyhow::Result<NostrNote> {
        let mut bot_state = self.lock().await;
        bot_state
            .admin_config
            .check_admin_whitelist(&admin_note.pubkey)?;
        let admin_req = AdminServerRequest::try_from(&admin_note)?;
        let update = match admin_req.config_type {
            AdminConfigurationType::ExchangeRate => {
                bot_state
                    .admin_config
                    .set_exchange_rate(admin_req.config_str.parse()?);
                bot_state.admin_config.sign_exchange_rate(signing_keys)?
            }
            AdminConfigurationType::CommerceWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&admin_req.config_str)?;
                bot_state.admin_config.set_commerce_whitelist(whitelist);
                bot_state
                    .admin_config
                    .sign_commerce_whitelist(signing_keys)?
            }
            AdminConfigurationType::CourierWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&admin_req.config_str)?;
                bot_state.admin_config.set_couriers_whitelist(whitelist);
                bot_state
                    .admin_config
                    .sign_couriers_whitelist(signing_keys)?
            }
            _ => return Err(anyhow!("Invalid config type")),
        };
        Ok(update)
    }
    pub async fn update_admin_config(
        &self,
        new_config: NostrNote,
        decrypted: Option<String>,
    ) -> anyhow::Result<()> {
        let mut bot_state = self.lock().await;
        let config_type: AdminConfigurationType = new_config
            .tags
            .find_tags(NostrTag::Parameterized)
            .get(2)
            .ok_or(anyhow!("No config type found"))?
            .clone()
            .try_into()?;
        match config_type {
            AdminConfigurationType::CommerceWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&new_config.content)?;
                info!("Commerce whitelist set to: {:?}", &bot_state.admin_config);
                bot_state.admin_config.set_commerce_whitelist(whitelist);
            }
            AdminConfigurationType::CourierWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&new_config.content)?;
                bot_state.admin_config.set_couriers_whitelist(whitelist);
            }
            AdminConfigurationType::ExchangeRate => {
                let rate: f64 = serde_json::from_str(&new_config.content)?;
                bot_state.admin_config.set_exchange_rate(rate);
                info!("Exchange rate set to: {}", rate);
            }
            AdminConfigurationType::ConsumerBlacklist => {
                let blacklist: Vec<String> = serde_json::from_str(&decrypted.unwrap())?;
                bot_state.admin_config.set_consumer_blacklist(blacklist);
            }
            AdminConfigurationType::UserRegistrations => {
                let registrations: Vec<String> = serde_json::from_str(&decrypted.unwrap())?;
                bot_state.admin_config.set_user_registrations(registrations);
            }
            AdminConfigurationType::AdminWhitelist => {
                let whitelist: Vec<String> = serde_json::from_str(&decrypted.unwrap())?;
                bot_state.admin_config.set_admin_whitelist(whitelist);
            }
        }
        Ok(())
    }
}
