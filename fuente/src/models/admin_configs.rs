use std::hash::{DefaultHasher, Hash, Hasher};

use nostr_minions::browser_api::IdbStoreManager;
#[cfg(target_arch = "wasm32")]
use nostr_minions::key_manager::UserIdentity;
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use super::TEST_PUB_KEY;

use super::{
    nostr_kinds::{NOSTR_KIND_ADMIN_REQUEST, NOSTR_KIND_SERVER_CONFIG},
    DB_NAME_FUENTE, DB_VERSION_FUENTE,
};

#[derive(Serialize, Deserialize, Clone, Hash)]
pub enum AdminConfigurationType {
    AdminWhitelist,
    CommerceWhitelist,
    ConsumerBlacklist,
    UserRegistrations,
    ExchangeRate,
    CourierWhitelist,
}
impl AdminConfigurationType {
    pub fn to_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }
}
impl TryFrom<&str> for AdminConfigurationType {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let i = value.parse::<u32>()?;
        match i {
            0 => Ok(AdminConfigurationType::AdminWhitelist),
            1 => Ok(AdminConfigurationType::CommerceWhitelist),
            2 => Ok(AdminConfigurationType::ConsumerBlacklist),
            3 => Ok(AdminConfigurationType::UserRegistrations),
            4 => Ok(AdminConfigurationType::ExchangeRate),
            5 => Ok(AdminConfigurationType::CourierWhitelist),
            _ => Err(anyhow::anyhow!("Invalid AdminConfigurationType")),
        }
    }
}
impl TryFrom<String> for AdminConfigurationType {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let i = value.parse::<u32>()?;
        match i {
            0 => Ok(AdminConfigurationType::AdminWhitelist),
            1 => Ok(AdminConfigurationType::CommerceWhitelist),
            2 => Ok(AdminConfigurationType::ConsumerBlacklist),
            3 => Ok(AdminConfigurationType::UserRegistrations),
            4 => Ok(AdminConfigurationType::ExchangeRate),
            5 => Ok(AdminConfigurationType::CourierWhitelist),
            _ => Err(anyhow::anyhow!("Invalid AdminConfigurationType")),
        }
    }
}
impl TryFrom<u32> for AdminConfigurationType {
    type Error = anyhow::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AdminConfigurationType::AdminWhitelist),
            1 => Ok(AdminConfigurationType::CommerceWhitelist),
            2 => Ok(AdminConfigurationType::ConsumerBlacklist),
            3 => Ok(AdminConfigurationType::UserRegistrations),
            4 => Ok(AdminConfigurationType::ExchangeRate),
            5 => Ok(AdminConfigurationType::CourierWhitelist),
            _ => Err(anyhow::anyhow!("Invalid AdminConfigurationType")),
        }
    }
}
impl From<AdminConfigurationType> for u32 {
    fn from(value: AdminConfigurationType) -> u32 {
        match value {
            AdminConfigurationType::AdminWhitelist => 0,
            AdminConfigurationType::CommerceWhitelist => 1,
            AdminConfigurationType::ConsumerBlacklist => 2,
            AdminConfigurationType::UserRegistrations => 3,
            AdminConfigurationType::ExchangeRate => 4,
            AdminConfigurationType::CourierWhitelist => 5,
        }
    }
}
impl Into<String> for AdminConfigurationType {
    fn into(self) -> String {
        let i = u32::from(self);
        i.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdminConfiguration {
    admin_whitelist: Vec<String>,
    commerce_whitelist: Vec<String>,
    couriers_whitelist: Vec<String>,
    consumer_blacklist: Vec<String>,
    user_registrations: Vec<String>,
    exchange_rate: f64,
}
impl Default for AdminConfiguration {
    fn default() -> Self {
        AdminConfiguration {
            admin_whitelist: Vec::new(),
            commerce_whitelist: Vec::new(),
            consumer_blacklist: Vec::new(),
            couriers_whitelist: Vec::new(),
            user_registrations: Vec::new(),
            exchange_rate: 1.0,
        }
    }
}
impl AdminConfiguration {
    pub fn sign_admin_whitelist(
        &self,
        priv_key: &NostrKeypair,
        receiver: String,
    ) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.admin_whitelist)?;
        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };
        let config_str: String = AdminConfigurationType::AdminWhitelist.into();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &receiver, &config_str));
        note.tags.add_parameter_tag(&receiver);
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nip_04_encrypted(&mut note, receiver)?;
        Ok(note)
    }
    pub fn sign_commerce_whitelist(&self, priv_key: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.commerce_whitelist)?;
        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };
        let config_str: String = AdminConfigurationType::CommerceWhitelist.into();
        let config_hash = AdminConfigurationType::CommerceWhitelist.to_hash();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &config_hash, &config_str));
        note.tags.add_parameter_tag(&config_hash.to_string());
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nostr_event(&mut note);
        Ok(note)
    }
    pub fn sign_couriers_whitelist(&self, priv_key: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.couriers_whitelist)?;

        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };
        let config_str: String = AdminConfigurationType::CourierWhitelist.into();
        let config_hash = AdminConfigurationType::CourierWhitelist.to_hash();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &config_hash, &config_str));
        note.tags.add_parameter_tag(&config_hash.to_string());
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nostr_event(&mut note);
        Ok(note)
    }
    pub fn sign_consumer_blacklist(&self, priv_key: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.consumer_blacklist)?;

        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };

        let config_str: String = AdminConfigurationType::ConsumerBlacklist.into();
        let config_hash = AdminConfigurationType::ConsumerBlacklist.to_hash();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &config_hash, &config_str));
        note.tags.add_parameter_tag(&config_hash.to_string());
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nostr_event(&mut note);
        Ok(note)
    }
    pub fn sign_user_registrations(
        &self,
        priv_key: &NostrKeypair,
        receiver: String,
    ) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.user_registrations)?;

        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };

        let config_str: String = AdminConfigurationType::UserRegistrations.into();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &receiver, &config_str));
        note.tags.add_parameter_tag(&receiver.to_string());
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nostr_event(&mut note);
        Ok(note)
    }
    pub fn sign_exchange_rate(&self, priv_key: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let serialized = serde_json::to_string(&self.exchange_rate)?;

        let mut note = NostrNote {
            pubkey: priv_key.public_key(),
            kind: NOSTR_KIND_SERVER_CONFIG,
            content: serialized,
            ..Default::default()
        };

        let config_str: String = AdminConfigurationType::ExchangeRate.into();
        let config_hash = AdminConfigurationType::ExchangeRate.to_hash();
        note.tags
            .add_parameter_tag(&format!("{}-{}", &config_hash, &config_str));
        note.tags.add_parameter_tag(&config_hash.to_string());
        note.tags.add_parameter_tag(&config_str);
        priv_key.sign_nostr_event(&mut note);
        Ok(note)
    }
    pub fn update_commerce_whitelist(&mut self, new_commerce: String) {
        self.commerce_whitelist.push(new_commerce);
    }
    pub fn set_admin_whitelist(&mut self, admin_whitelist: Vec<String>) {
        self.admin_whitelist = admin_whitelist;
    }
    pub fn set_commerce_whitelist(&mut self, commerce_whitelist: Vec<String>) {
        self.commerce_whitelist = commerce_whitelist;
    }
    pub fn set_consumer_blacklist(&mut self, consumer_blacklist: Vec<String>) {
        self.consumer_blacklist = consumer_blacklist;
    }
    pub fn set_couriers_whitelist(&mut self, couriers_whitelist: Vec<String>) {
        self.couriers_whitelist = couriers_whitelist;
    }
    pub fn set_user_registrations(&mut self, user_registrations: Vec<String>) {
        self.user_registrations = user_registrations;
    }
    pub fn set_exchange_rate(&mut self, exchange_rate: f64) {
        self.exchange_rate = exchange_rate;
    }
    pub fn check_admin_whitelist(&self, admin: &str) -> anyhow::Result<()> {
        if self.admin_whitelist.contains(&admin.to_string()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Admin not in whitelist"))
        }
    }
    pub fn check_couriers_whitelist(&self, courier: &str) -> anyhow::Result<()> {
        if self.couriers_whitelist.contains(&courier.to_string()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!(format!(
                "Courier not in whitelist {}",
                courier
            )))
        }
    }
    pub fn check_commerce_whitelist(&self, commerce: &str) -> anyhow::Result<()> {
        if self.commerce_whitelist.contains(&commerce.to_string()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!(format!(
                "Commerce not in whitelist {}",
                commerce
            )))
        }
    }
    pub fn check_consumer_blacklist(&self, consumer: &str) -> anyhow::Result<()> {
        if self.consumer_blacklist.contains(&consumer.to_string()) {
            Err(anyhow::anyhow!("Consumer in blacklist"))
        } else {
            Ok(())
        }
    }
    pub fn check_user_registrations(&self, user: &str) -> anyhow::Result<()> {
        if self.user_registrations.contains(&user.to_string()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not in registrations"))
        }
    }
    pub fn get_exchange_rate(&self) -> f64 {
        self.exchange_rate
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminServerRequest {
    pub config_type: AdminConfigurationType,
    pub config_str: String,
}
impl AdminServerRequest {
    pub fn new(config_type: AdminConfigurationType, config_str: String) -> Self {
        AdminServerRequest {
            config_type,
            config_str,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub async fn sign_data(&self, priv_key: &UserIdentity) -> anyhow::Result<NostrNote> {
        let pubkey = priv_key
            .get_pubkey()
            .await
            .ok_or(anyhow::anyhow!("No pubkey"))?;
        let mut note = NostrNote {
            pubkey: pubkey.clone(),
            kind: NOSTR_KIND_ADMIN_REQUEST,
            content: self.config_str.clone(),
            ..Default::default()
        };
        let config_str: String = self.config_type.clone().into();
        note.tags.add_parameter_tag(&config_str);
        let note = priv_key
            .sign_nostr_note(note)
            .await
            .map_err(|_e| anyhow::anyhow!("Could not sign note"))?;
        let giftwrap = NostrNote {
            pubkey,
            kind: NOSTR_KIND_ADMIN_REQUEST,
            content: note.into(),
            ..Default::default()
        };
        priv_key
            .sign_nip44(giftwrap, TEST_PUB_KEY.to_string())
            .await
            .map_err(|_e| anyhow::anyhow!("Could not sign giftwrap"))
    }
}
impl ToString for AdminServerRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<&str> for AdminServerRequest {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let req: AdminServerRequest = serde_json::from_str(value)?;
        Ok(req)
    }
}
impl TryFrom<String> for AdminServerRequest {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let req: AdminServerRequest = serde_json::from_str(&value)?;
        Ok(req)
    }
}
impl TryFrom<&NostrNote> for AdminServerRequest {
    type Error = anyhow::Error;
    fn try_from(value: &NostrNote) -> Result<Self, Self::Error> {
        if value.kind != NOSTR_KIND_ADMIN_REQUEST {
            return Err(anyhow::anyhow!("Invalid kind"));
        }
        let config_str = value.content.clone();
        let config_type = value
            .tags
            .find_first_parameter()
            .ok_or(anyhow::anyhow!("No config type"))?
            .clone();
        let config_type = AdminConfigurationType::try_from(config_type)?;
        Ok(AdminServerRequest::new(config_type, config_str))
    }
}
impl TryFrom<NostrNote> for AdminServerRequest {
    type Error = anyhow::Error;
    fn try_from(value: NostrNote) -> Result<Self, Self::Error> {
        if value.kind != NOSTR_KIND_ADMIN_REQUEST {
            return Err(anyhow::anyhow!("Invalid kind"));
        }
        let config_str = value.content;
        let config_type = value
            .tags
            .find_first_parameter()
            .ok_or(anyhow::anyhow!("No config type"))?
            .clone();
        let config_type = AdminConfigurationType::try_from(config_type)?;
        Ok(AdminServerRequest::new(config_type, config_str))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlatformStatIdb {
    note_id: String,
    state_note: NostrNote,
}
impl Default for PlatformStatIdb {
    fn default() -> Self {
        let note_id = NostrKeypair::generate(false).public_key();
        let order = NostrNote::default();
        Self {
            note_id,
            state_note: order,
        }
    }
}
impl Into<JsValue> for PlatformStatIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for PlatformStatIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl PlatformStatIdb {
    pub fn new(order: NostrNote) -> Result<Self, JsValue> {
        Ok(Self {
            note_id: order.id.clone().ok_or(JsValue::from_str("No id"))?,
            state_note: order,
        })
    }
    pub async fn find_history() -> Result<Vec<NostrNote>, JsValue> {
        let db_entries = Self::retrieve_all_from_store().await?;
        let order_states = db_entries
            .iter()
            .map(|entry| entry.signed_note())
            .collect::<Vec<NostrNote>>();
        Ok(order_states)
    }
    pub async fn save(&self) -> Result<(), JsValue> {
        self.clone().save_to_store().await
    }
    pub fn signed_note(&self) -> NostrNote {
        self.state_note.clone()
    }
    pub fn id(&self) -> String {
        self.state_note.id.clone().unwrap_or_default()
    }
    pub async fn last_saved_timestamp() -> anyhow::Result<i64> {
        let db_entries = Self::retrieve_all_from_store()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let last_entry = db_entries
            .iter()
            .max_by(|a, b| a.state_note.created_at.cmp(&b.state_note.created_at))
            .ok_or_else(|| anyhow::anyhow!("No entries found"))?;
        Ok(last_entry.state_note.created_at)
    }
}
impl IdbStoreManager for PlatformStatIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: "stats",
            document_key: "note_id",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.state_note.id.clone().unwrap_or_default())
    }
}
