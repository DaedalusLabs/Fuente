use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use super::{
    nostr_kinds::{
        NOSTR_KIND_CONSUMER_GIFTWRAP, NOSTR_KIND_DRIVER_PROFILE, NOSTR_KIND_DRIVER_STATE,
    },
    DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_PROFILES,
};
use nostr_minions::browser_api::{GeolocationPosition, IdbStoreManager};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DriverProfile {
    nickname: String,
    telephone: String,
}
impl Default for DriverProfile {
    fn default() -> Self {
        Self {
            nickname: "John Doe".to_string(),
            telephone: "11111111".to_string(),
        }
    }
}
impl ToString for DriverProfile {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl From<JsValue> for DriverProfile {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}
impl Into<JsValue> for DriverProfile {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<&str> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<String> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<NostrNote> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_DRIVER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: DriverProfile = note.content.try_into()?;
        Ok(profile)
    }
}
impl DriverProfile {
    pub fn new(nickname: String, telephone: String) -> Self {
        Self {
            nickname,
            telephone,
        }
    }
    pub fn signed_data(&self, keys: &NostrKeypair) -> NostrNote {
        let mut new_note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_DRIVER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_event(&mut new_note);
        new_note
    }
    pub fn giftwrapped_data(
        &self,
        keys: &NostrKeypair,
        receiver: String,
    ) -> anyhow::Result<NostrNote> {
        let mut unsigned_note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_DRIVER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_event(&mut unsigned_note);
        let mut giftwrap = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_CONSUMER_GIFTWRAP,
            content: unsigned_note.to_string(),
            ..Default::default()
        };
        keys.sign_nip_04_encrypted(&mut giftwrap, receiver)?;
        Ok(giftwrap)
    }
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }
    pub fn telephone(&self) -> String {
        self.telephone.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStateUpdate {
    driver: NostrNote,
    geolocation: GeolocationPosition,
}
impl DriverStateUpdate {
    pub async fn new(driver: NostrNote) -> anyhow::Result<Self> {
        let _: DriverProfile = driver.clone().try_into()?;
        let geo = GeolocationPosition::locate()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self {
            driver,
            geolocation: geo,
        })
    }
    pub fn sign_update(
        &self,
        user_keys: &NostrKeypair,
        recipient: String,
    ) -> anyhow::Result<NostrNote> {
        let mut new_note = NostrNote {
            pubkey: user_keys.public_key(),
            kind: NOSTR_KIND_DRIVER_STATE,
            content: self.to_string(),
            ..Default::default()
        };
        user_keys.sign_nip_04_encrypted(&mut new_note, recipient)?;
        Ok(new_note)
    }
    pub fn from_signed_update(
        &self,
        user_keys: &NostrKeypair,
        signed_note: NostrNote,
    ) -> anyhow::Result<Self> {
        let note = user_keys.decrypt_nip_04_content(&signed_note)?;
        let update: DriverStateUpdate = note.try_into()?;
        Ok(update)
    }
}
impl ToString for DriverStateUpdate {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for DriverStateUpdate {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Into<JsValue> for DriverStateUpdate {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for DriverStateUpdate {
    type Error = anyhow::Error;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value).map_err(|e| anyhow::anyhow!("{:?}", e))?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverProfileIdb {
    pubkey: String,
    note: NostrNote,
    profile: DriverProfile,
}

impl DriverProfileIdb {
    pub fn new(profile: DriverProfile, keys: &NostrKeypair) -> Self {
        let pubkey = keys.public_key().to_string();
        let note = profile.signed_data(keys);
        Self {
            pubkey,
            note,
            profile,
        }
    }
    pub fn signed_note(&self) -> NostrNote {
        self.note.clone()
    }
    pub fn profile(&self) -> DriverProfile {
        self.profile.clone()
    }
    pub fn pubkey(&self) -> String {
        self.pubkey.clone()
    }
}

impl Into<JsValue> for DriverProfileIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

impl TryFrom<JsValue> for DriverProfileIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

impl TryFrom<NostrNote> for DriverProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_DRIVER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let pubkey = note.pubkey.to_string();
        let profile: DriverProfile = note.clone().try_into()?;
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for DriverProfileIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_CONSUMER_PROFILES,
            document_key: "pubkey",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::init_consumer_db;
    use nostr_minions::browser_api::IdbStoreManager;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn _commerce_profile_idb() -> Result<(), JsValue> {
        init_consumer_db()?;
        let key_1 = NostrKeypair::generate(false);
        let consumer_address = DriverProfile::default();
        let address_idb = DriverProfileIdb::new(consumer_address.clone(), &key_1);
        address_idb.clone().save_to_store().await.unwrap();

        let key_2 = NostrKeypair::generate(false);
        let address_idb_2 = DriverProfileIdb::new(consumer_address, &key_2);
        address_idb_2.clone().save_to_store().await.unwrap();

        let retrieved: DriverProfileIdb = DriverProfileIdb::retrieve_from_store(&address_idb.key())
            .await
            .unwrap();
        assert_eq!(retrieved.pubkey(), address_idb.pubkey());

        let retrieved_2: DriverProfileIdb =
            DriverProfileIdb::retrieve_from_store(&address_idb_2.key())
                .await
                .unwrap();
        assert_eq!(retrieved_2.pubkey(), address_idb_2.pubkey());

        let all_addresses = DriverProfileIdb::retrieve_all_from_store().await.unwrap();
        assert_eq!(all_addresses.len(), 2);

        let deleted = retrieved.delete_from_store().await;
        let deleted_2 = retrieved_2.delete_from_store().await;
        assert!(deleted.is_ok());
        assert!(deleted_2.is_ok());
        Ok(())
    }
}
