use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsValue;

use super::{
    nostr_kinds::{NOSTR_KIND_COURIER_PROFILE, NOSTR_KIND_DRIVER_STATE},
    DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_PROFILES,
};
use nostr_minions::{
    browser_api::{GeolocationCoordinates, GeolocationPosition, IdbStoreManager},
    key_manager::UserIdentity,
};

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
impl TryFrom<&NostrNote> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(note: &NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COURIER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: DriverProfile = note.content.as_str().try_into()?;
        Ok(profile)
    }
}
impl TryFrom<NostrNote> for DriverProfile {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COURIER_PROFILE {
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
    pub async fn signed_data(&self, keys: &UserIdentity) -> NostrNote {
        let pubkey = keys.get_pubkey().await.expect("no pubkey");
        let new_note = NostrNote {
            pubkey,
            kind: NOSTR_KIND_COURIER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_note(new_note)
            .await
            .expect("could not sign")
    }
    pub async fn giftwrapped_data(
        &self,
        keys: &UserIdentity,
        receiver: String,
        tag: String,
    ) -> anyhow::Result<NostrNote> {
        let pubkey = keys
            .get_pubkey()
            .await
            .ok_or(anyhow::anyhow!("No pubkey"))?;
        let note = NostrNote {
            pubkey: pubkey.clone(),
            kind: NOSTR_KIND_COURIER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        let note = keys.sign_nostr_note(note).await.map_err(|e| {
            anyhow::anyhow!(e.as_string().unwrap_or_else(|| "Unknown error".to_string()))
        })?;
        let mut giftwrap = NostrNote {
            pubkey,
            kind: NOSTR_KIND_COURIER_PROFILE,
            content: note.to_string(),
            ..Default::default()
        };
        giftwrap.tags.add_parameter_tag(&tag);
        let giftwrap = keys.sign_nip44(giftwrap, receiver).await.map_err(|e| {
            anyhow::anyhow!(e.as_string().unwrap_or_else(|| "Unknown error".to_string()))
        })?;
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
    // The new method looks fine, just fix the formatting and wildcards
    pub async fn new(driver: NostrNote) -> anyhow::Result<Self> {
        let geo = GeolocationPosition::locate().await.map_err(|e| {
            anyhow::anyhow!(e.as_string().unwrap_or_else(|| "Unknown error".to_string()))
        })?;
        Ok(Self {
            driver,
            geolocation: geo,
        })
    }

    pub async fn to_encrypted_note(
        &self,
        keys: &UserIdentity,
        recipient: String,
    ) -> anyhow::Result<NostrNote> {
        // Convert state directly to string - no nesting
        let pubkey = keys
            .get_pubkey()
            .await
            .ok_or(anyhow::anyhow!("No pubkey"))?;
        let location_data = serde_json::to_string(&self)?;

        // Create a single encrypted note
        let encrypted_note = NostrNote {
            kind: NOSTR_KIND_DRIVER_STATE,
            content: location_data, // Direct location data
            pubkey,
            ..Default::default()
        };

        // Encrypt it for the recipient
        keys.sign_nip44(encrypted_note, recipient)
            .await
            .map_err(|e| {
                anyhow::anyhow!(e.as_string().unwrap_or_else(|| "Unknown error".to_string()))
            })
    }

    // pub fn from_signed_update(
    //     &self,
    //     user_keys: &NostrKeypair,
    //     signed_note: NostrNote,
    // ) -> anyhow::Result<Self> {
    //     let note = user_keys.decrypt_nip_04_content(&signed_note)?;
    //     let update: DriverStateUpdate = note.try_into()?;
    //     Ok(update)
    // }
    pub fn get_location(&self) -> GeolocationCoordinates {
        self.geolocation.coords.clone()
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
    pub async fn new(profile: DriverProfile, keys: &UserIdentity) -> Self {
        let pubkey = keys.get_pubkey().await.expect("no pubkey");
        let note = profile.signed_data(keys).await;
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
        if note.kind != NOSTR_KIND_COURIER_PROFILE {
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
