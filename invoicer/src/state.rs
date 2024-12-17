use std::sync::Arc;

use anyhow::anyhow;
use fuente::models::{
    AdminConfiguration, AdminConfigurationType, AdminServerRequest, CommerceProfile,
    OrderInvoiceState, ProductMenu, TEST_PUB_KEY,
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
    pub fn new() -> Self {
        let mut admin_config = AdminConfiguration::default();
        // TODO
        // make this env variables
        admin_config.set_admin_whitelist(vec![TEST_PUB_KEY.to_string()]);
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
    pub async fn check_courier_whitelist(&self, pubkey: &str) -> anyhow::Result<()> {
        self.lock()
            .await
            .admin_config
            .check_couriers_whitelist(pubkey)
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
    pub async fn add_live_order(&self, order: OrderInvoiceState) -> anyhow::Result<()> {
        let mut orders = self.lock().await;
        orders.live_orders.new_order(order.id(), order)?;
        Ok(())
    }
    pub async fn handle_courier_updates(
        &self,
        inner_note: NostrNote,
        outer_note: NostrNote,
    ) -> anyhow::Result<OrderInvoiceState> {
        let bot_state = self.lock_owned().await;
        bot_state
            .admin_config
            .check_couriers_whitelist(&inner_note.pubkey)?;
        let updated_order: OrderInvoiceState =
            OrderInvoiceState::try_from(inner_note.content.to_string())?;
        // Check if order is part of live orders
        let mut live_order = bot_state
            .live_orders
            .get_order(&updated_order.id())
            .ok_or(anyhow!("Order not found"))?;
        // Check if the courier is already assigned
        if live_order.get_courier().is_none() {
            let new_courier = updated_order
                .get_courier()
                .ok_or(anyhow!("No courier found"))?;
            live_order.update_courier(new_courier);
            return Ok(live_order.to_owned());
        }
        // Check if the update s coing from asigned courier
        if outer_note.pubkey == live_order.get_courier().unwrap().pubkey {
            info!("Order updated by courier");
            return Ok(updated_order.to_owned());
        }
        Err(anyhow!("Invalid courier update"))
    }
    pub async fn sign_updated_config(
        &self,
        admin_req: AdminServerRequest,
        signing_keys: &NostrKeypair,
    ) -> anyhow::Result<NostrNote> {
        let mut bot_state = self.lock().await;
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
        bot_state
            .admin_config
            .check_admin_whitelist(&new_config.pubkey)?;
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
                bot_state.admin_config.set_commerce_whitelist(whitelist);
                info!("Commerce whitelist updated");
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
