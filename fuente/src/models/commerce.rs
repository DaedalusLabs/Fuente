use crate::browser::{indexed_db::IdbStoreManager, nominatim::NominatimLookup};

use super::{
    gps::CoordinateStrings, nostr_kinds::NOSTR_KIND_COMMERCE_PROFILE, upgrade_shared_db,
    DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_COMMERCE_PROFILES,
};
use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::browser::geolocation::GeolocationCoordinates;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommerceProfile {
    name: String,
    description: String,
    telephone: String,
    web: String,
    lookup: NominatimLookup,
    geolocation: CoordinateStrings,
}

impl CommerceProfile {
    pub fn new(
        name: String,
        description: String,
        telephone: String,
        web: String,
        lookup: NominatimLookup,
        geo: GeolocationCoordinates,
    ) -> Self {
        Self {
            name,
            description,
            telephone,
            web,
            lookup,
            geolocation: geo.into(),
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
    id: String,
    note: SignedNote,
    profile: CommerceProfile,
}

impl CommerceProfileIdb {
    pub fn new(profile: CommerceProfile, keys: &UserKeys) -> Result<Self, JsValue> {
        let note = profile.signed_data(keys);
        let id = note.get_pubkey().to_string();
        Ok(Self { id, note, profile })
    }
    pub async fn save(self) -> Result<(), JsValue> {
        self.save_to_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn delete(&self) -> Result<(), JsValue> {
        self.delete_from_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn find(id: &str) -> Result<Self, JsValue> {
        Self::retrieve::<Self>(id)?
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub async fn find_all() -> Result<Vec<Self>, JsValue> {
        Self::retrieve_all_from_store::<Self>()?
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn profile(&self) -> &CommerceProfile {
        &self.profile
    }
    pub fn signed_note(&self) -> &SignedNote {
        &self.note
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl TryFrom<JsValue> for CommerceProfileIdb {
    type Error = JsValue;
    fn try_from(js_value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(js_value)?)
    }
}

impl TryInto<JsValue> for CommerceProfileIdb {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}

impl TryFrom<SignedNote> for CommerceProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_COMMERCE_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: CommerceProfile = note.get_content().try_into()?;
        let id = note.get_pubkey().to_string();
        Ok(CommerceProfileIdb { id, note, profile })
    }
}

impl IdbStoreManager for CommerceProfileIdb {
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }

    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }

    fn store_name() -> &'static str {
        STORE_NAME_COMMERCE_PROFILES
    }

    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }

    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_shared_db(event)
    }
}
