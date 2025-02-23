use bright_lightning::LightningAddress;
use nostr_minions::{
    browser_api::{GeolocationCoordinates, IdbStoreManager},
    key_manager::UserIdentity,
    widgets::leaflet::nominatim::NominatimLookup,
};

use super::{
    gps::CoordinateStrings, nostr_kinds::NOSTR_KIND_COMMERCE_PROFILE, DB_NAME_FUENTE,
    DB_VERSION_FUENTE, STORE_NAME_COMMERCE_PROFILES,
};
use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommerceProfile {
    pub name: String,
    pub description: String,
    pub telephone: String,
    pub web: String,
    pub lookup: NominatimLookup,
    pub geolocation: CoordinateStrings,
    pub ln_address: String,
    pub logo_url: String,
    pub banner_url: String,
}
impl Default for CommerceProfile {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            description: "".to_string(),
            telephone: "".to_string(),
            web: "".to_string(),
            lookup: NominatimLookup::default(),
            geolocation: CoordinateStrings::default(),
            ln_address: "".to_string(),
            logo_url: "".to_string(),
            banner_url: "".to_string(),
        }
    }
}
impl CommerceProfile {
    pub fn new(
        name: String,
        description: String,
        telephone: String,
        web: String,
        lookup: NominatimLookup,
        geo: GeolocationCoordinates,
        ln_address: String,
        logo_url: String,
        banner_url: String,
    ) -> Self {
        Self {
            name,
            description,
            telephone,
            web,
            lookup,
            geolocation: geo.into(),
            ln_address,
            logo_url,
            banner_url,
        }
    }
    pub async fn signed_data(&self, user_keys: &UserIdentity) -> NostrNote {
        let pubkey = user_keys.get_pubkey().await.unwrap();
        let data = serde_json::to_string(self).unwrap();
        let new_note = NostrNote {
            pubkey,
            kind: NOSTR_KIND_COMMERCE_PROFILE,
            content: data,
            ..Default::default()
        };
        user_keys
            .sign_nostr_note(new_note)
            .await
            .expect("Failed to sign note")
    }
    pub fn geolocation(&self) -> GeolocationCoordinates {
        self.geolocation.clone().into()
    }
    pub fn ln_address(&self) -> LightningAddress {
        let address = Box::leak(self.ln_address.clone().into_boxed_str());
        LightningAddress(address)
    }
}
impl TryFrom<String> for CommerceProfile {
    type Error = anyhow::Error;
    fn try_from(json: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&json)?)
    }
}
impl ToString for CommerceProfile {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<NostrNote> for CommerceProfile {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COMMERCE_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let details: CommerceProfile = serde_json::from_str(&note.content)?;
        Ok(details)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommerceProfileIdb {
    pubkey: String,
    note: NostrNote,
    profile: CommerceProfile,
}

impl CommerceProfileIdb {
    pub async fn new(profile: CommerceProfile, keys: &UserIdentity) -> Result<Self, JsValue> {
        let note = profile.signed_data(keys).await;
        let pubkey = note.pubkey.clone();
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
    pub fn profile(&self) -> &CommerceProfile {
        &self.profile
    }
    pub fn signed_note(&self) -> &NostrNote {
        &self.note
    }
    pub fn id(&self) -> &str {
        &self.pubkey
    }
    pub fn idb_key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
}

impl TryFrom<JsValue> for CommerceProfileIdb {
    type Error = JsValue;
    fn try_from(js_value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(js_value)?)
    }
}

impl Into<JsValue> for CommerceProfileIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

impl TryFrom<NostrNote> for CommerceProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_COMMERCE_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: CommerceProfile = note.content.clone().try_into()?;
        let pubkey = note.pubkey.clone();
        Ok(CommerceProfileIdb {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for CommerceProfileIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_COMMERCE_PROFILES,
            document_key: "pubkey",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
}
