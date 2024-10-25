// Admins need to control the configuration of the server
// This includes the following:
// 1. Admin whitelist
// 2. Commerces whitelist
// 3. Consumer blacklist
// 4. User registrations
// 5. Exchange rates

use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};

use super::nostr_kinds::NOSTR_KIND_SERVER_CONFIG;

#[derive(Serialize, Deserialize, Clone)]
pub enum AdminConfigurationType {
    AdminWhitelist,
    CommerceWhitelist,
    ConsumerBlacklist,
    UserRegistrations,
    ExchangeRate,
    CourierWhitelist,
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

#[derive(Serialize, Deserialize, Clone)]
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
    pub fn sign_admin_whitelist(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.admin_whitelist)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::AdminWhitelist.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
    }
    pub fn sign_commerce_whitelist(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.commerce_whitelist)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::CommerceWhitelist.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
    }
    pub fn sign_couriers_whitelist(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.couriers_whitelist)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::CommerceWhitelist.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
    }
    pub fn sign_consumer_blacklist(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.consumer_blacklist)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::ConsumerBlacklist.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
    }
    pub fn sign_user_registrations(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.user_registrations)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::UserRegistrations.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
    }
    pub fn sign_exchange_rate(&self, priv_key: &UserKeys) -> anyhow::Result<SignedNote> {
        let serialized = serde_json::to_string(&self.exchange_rate)?;
        let pubkey = priv_key.get_public_key();
        let mut note = Note::new(&pubkey, NOSTR_KIND_SERVER_CONFIG, &serialized);
        let config_str: String = AdminConfigurationType::ExchangeRate.into();
        note.add_tag("d", &config_str);
        priv_key.sign_nip_04_encrypted(note, pubkey)
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
            Err(anyhow::anyhow!("Courier not in whitelist"))
        }
    }
    pub fn check_commerce_whitelist(&self, commerce: &str) -> anyhow::Result<()> {
        if self.commerce_whitelist.contains(&commerce.to_string()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Commerce not in whitelist"))
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
