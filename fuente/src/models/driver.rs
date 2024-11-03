use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use super::{
    nostr_kinds::{
        NOSTR_KIND_CONSUMER_GIFTWRAP, NOSTR_KIND_DRIVER_PROFILE, NOSTR_KIND_DRIVER_STATE,
    }, upgrade_fuente_db, DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_PROFILES
};
use crate::browser_api::{GeolocationPosition, IdbStoreManager};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DriverProfile {
    nickname: String,
    telephone: String,
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
impl TryFrom<SignedNote> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_DRIVER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: DriverProfile = note.get_content().try_into()?;
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
    pub fn signed_data(&self, keys: &UserKeys) -> SignedNote {
        let unsigned_note = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_DRIVER_PROFILE,
            &self.to_string(),
        );
        keys.sign_nostr_event(unsigned_note)
    }
    pub fn giftwrapped_data(
        &self,
        keys: &UserKeys,
        receiver: String,
    ) -> anyhow::Result<SignedNote> {
        let unsigned_note = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_DRIVER_PROFILE,
            &self.to_string(),
        );
        let signed_profile = keys.sign_nostr_event(unsigned_note);
        let giftwrap = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_CONSUMER_GIFTWRAP,
            &signed_profile.to_string(),
        );
        keys.sign_nip_04_encrypted(giftwrap, receiver)
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
    driver: SignedNote,
    geolocation: GeolocationPosition,
}
impl DriverStateUpdate {
    pub async fn new(driver: SignedNote) -> anyhow::Result<Self> {
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
        user_keys: &UserKeys,
        recipient: String,
    ) -> anyhow::Result<SignedNote> {
        let unsigned_note = Note::new(
            &user_keys.get_public_key(),
            NOSTR_KIND_DRIVER_STATE,
            &self.to_string(),
        );
        user_keys.sign_nip_04_encrypted(unsigned_note, recipient)
    }
    pub fn from_signed_update(
        &self,
        user_keys: &UserKeys,
        signed_note: SignedNote,
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
    note: SignedNote,
    profile: DriverProfile,
}

impl DriverProfileIdb {
    pub fn new(profile: DriverProfile, keys: &UserKeys) -> Self {
        let pubkey = keys.get_public_key().to_string();
        let note = profile.signed_data(keys);
        Self {
            pubkey,
            note,
            profile,
        }
    }
    pub fn signed_note(&self) -> SignedNote {
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

impl TryFrom<SignedNote> for DriverProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_DRIVER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let pubkey = note.get_pubkey().to_string();
        let profile: DriverProfile = note.clone().try_into()?;
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for DriverProfileIdb {
    fn config() -> crate::browser_api::IdbStoreConfig {
        crate::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_CONSUMER_PROFILES,
            document_key: "pubkey",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
    fn upgrade_db(db: web_sys::IdbDatabase) -> Result<(), JsValue> {
        upgrade_fuente_db(db)?;
        Ok(())
    }
}
