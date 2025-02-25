use bright_lightning::LightningAddress;
use nostr_minions::{
    browser_api::{GeolocationCoordinates, IdbStoreManager},
    widgets::leaflet::nominatim::NominatimLookup,
};

use super::{
    gps::CoordinateStrings, nostr_kinds::NOSTR_KIND_COMMERCE_PROFILE, DB_NAME_FUENTE,
    DB_VERSION_FUENTE, STORE_NAME_COMMERCE_PROFILES,
};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

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
    pub fn signed_data(&self, user_keys: &NostrKeypair) -> NostrNote {
        let data = serde_json::to_string(self).unwrap();
        let mut new_note = NostrNote {
            pubkey: user_keys.public_key(),
            kind: NOSTR_KIND_COMMERCE_PROFILE,
            content: data,
            ..Default::default()
        };
        user_keys.sign_nostr_event(&mut new_note);
        new_note
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
    pub fn new(profile: CommerceProfile, keys: &NostrKeypair) -> Result<Self, JsValue> {
        let note = profile.signed_data(keys);
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
        let consumer_address = CommerceProfile::default();
        let address_idb = CommerceProfileIdb::new(consumer_address.clone(), &key_1)?;
        address_idb.clone().save_to_store().await.unwrap();

        let key_2 = NostrKeypair::generate(false);
        let address_idb_2 = CommerceProfileIdb::new(consumer_address, &key_2)?;
        address_idb_2.clone().save_to_store().await.unwrap();

        let retrieved: CommerceProfileIdb =
            CommerceProfileIdb::retrieve_from_store(&address_idb.key())
                .await
                .unwrap();
        assert_eq!(retrieved.id(), address_idb.id());

        let retrieved_2: CommerceProfileIdb =
            CommerceProfileIdb::retrieve_from_store(&address_idb_2.key())
                .await
                .unwrap();
        assert_eq!(retrieved_2.id(), address_idb_2.id());

        let all_addresses = CommerceProfileIdb::retrieve_all_from_store().await.unwrap();
        assert_eq!(all_addresses.len(), 2);

        let deleted = retrieved.delete_from_store().await;
        let deleted_2 = retrieved_2.delete_from_store().await;
        assert!(deleted.is_ok());
        assert!(deleted_2.is_ok());
        Ok(())
    }
}
