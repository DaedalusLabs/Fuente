use crate::{
    browser_api::{GeolocationCoordinates, IdbStoreManager},
    widgets::leaflet::NominatimLookup,
};

use super::{
    gps::CoordinateStrings, nostr_kinds::NOSTR_KIND_COMMERCE_PROFILE, upgrade_fuente_db,
    DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_COMMERCE_PROFILES,
};
use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommerceProfile {
    name: String,
    description: String,
    telephone: String,
    web: String,
    lookup: NominatimLookup,
    geolocation: CoordinateStrings,
    ln_address: String,
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
    ) -> Self {
        Self {
            name,
            description,
            telephone,
            web,
            lookup,
            geolocation: geo.into(),
            ln_address,
        }
    }
    pub fn signed_data(&self, user_keys: &UserKeys) -> SignedNote {
        let data = serde_json::to_string(self).unwrap();
        let new_note = Note::new(
            &user_keys.get_public_key(),
            NOSTR_KIND_COMMERCE_PROFILE,
            &data,
        );
        user_keys.sign_nostr_event(new_note)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn telephone(&self) -> &str {
        &self.telephone
    }
    pub fn web(&self) -> &str {
        &self.web
    }
    pub fn geolocation(&self) -> GeolocationCoordinates {
        self.geolocation.clone().into()
    }
    pub fn ln_address(&self) -> &str {
        &self.ln_address
    }
    pub fn lookup(&self) -> &NominatimLookup {
        &self.lookup
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
impl TryFrom<SignedNote> for CommerceProfile {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_COMMERCE_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let details: CommerceProfile = serde_json::from_str(&note.get_content())?;
        Ok(details)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommerceProfileIdb {
    pubkey: String,
    note: SignedNote,
    profile: CommerceProfile,
}

impl CommerceProfileIdb {
    pub fn new(profile: CommerceProfile, keys: &UserKeys) -> Result<Self, JsValue> {
        let note = profile.signed_data(keys);
        let pubkey = note.get_pubkey();
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
    pub fn profile(&self) -> &CommerceProfile {
        &self.profile
    }
    pub fn signed_note(&self) -> &SignedNote {
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

impl TryFrom<SignedNote> for CommerceProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_COMMERCE_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: CommerceProfile = note.get_content().try_into()?;
        let pubkey = note.get_pubkey();
        Ok(CommerceProfileIdb {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for CommerceProfileIdb {
    fn config() -> crate::browser_api::IdbStoreConfig {
        crate::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_COMMERCE_PROFILES,
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
