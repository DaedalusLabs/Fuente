use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use super::{
    nostr_kinds::{NOSTR_KIND_DRIVER_PROFILE, NOSTR_KIND_DRIVER_STATE},
    upgrade_shared_db, DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_CONSUMER_PROFILES,
};
use crate::browser::{geolocation::GeolocationPosition, indexed_db::IdbStoreManager};

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
        let geo = GeolocationPosition::get_current_position().await?;
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
    id: String,
    note: SignedNote,
    profile: DriverProfile,
}

impl DriverProfileIdb {
    pub fn new(profile: DriverProfile, keys: &UserKeys) -> Self {
        let id = keys.get_public_key().to_string();
        let note = profile.signed_data(keys);
        Self { id, note, profile }
    }
    pub async fn save(self) -> anyhow::Result<()> {
        Ok(self
            .save_to_store()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .await?)
    }
    pub async fn delete(&self) -> anyhow::Result<()> {
        Ok(self
            .delete_from_store()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .await?)
    }
    pub fn signed_note(&self) -> SignedNote {
        self.note.clone()
    }
    pub fn profile(&self) -> DriverProfile {
        self.profile.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub async fn find_profile(id: &str) -> anyhow::Result<Self> {
        Ok(Self::retrieve::<Self>(id)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .await?)
    }
    pub async fn find_all_profiles() -> anyhow::Result<Vec<Self>> {
        Ok(Self::retrieve_all_from_store::<Self>()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .await?)
    }
}

impl TryInto<JsValue> for DriverProfileIdb {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
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
        let id = note.get_pubkey().to_string();
        let profile: DriverProfile = note.clone().try_into()?;
        Ok(Self { id, note, profile })
    }
}

impl IdbStoreManager for DriverProfileIdb {
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }

    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }

    fn store_name() -> &'static str {
        STORE_NAME_CONSUMER_PROFILES
    }

    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }

    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_shared_db(event)
    }
}
