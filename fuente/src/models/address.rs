use super::NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP;
use nostr_minions::{
    browser_api::IdbStoreManager, key_manager::UserIdentity,
    widgets::leaflet::nominatim::NominatimLookup,
};
use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use web_sys::wasm_bindgen::JsValue;

use super::{
    gps::CoordinateStrings, nostr_kinds::NOSTR_KIND_CONSUMER_PROFILE_ADDRESS, DB_NAME_FUENTE,
    DB_VERSION_FUENTE, STORE_NAME_CONSUMER_ADDRESSES,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ConsumerAddress {
    lookup: NominatimLookup,
    coordinates: CoordinateStrings,
}
impl ConsumerAddress {
    pub fn new(lookup: NominatimLookup, coordinates: CoordinateStrings) -> Self {
        Self {
            lookup,
            coordinates,
        }
    }
    pub fn lookup(&self) -> NominatimLookup {
        self.lookup.clone()
    }
    pub fn coordinates(&self) -> CoordinateStrings {
        self.coordinates.clone()
    }
    pub async fn signed_data(&self, keys: &UserIdentity) -> NostrNote {
        let new_note = NostrNote {
            pubkey: keys.get_pubkey().await.unwrap(),
            kind: NOSTR_KIND_CONSUMER_PROFILE_ADDRESS,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_note(new_note)
            .await
            .expect("Failed to sign note")
    }
    pub async fn giftwrapped_data(
        &self,
        keys: &UserIdentity,
        recipient: String,
    ) -> Result<NostrNote, JsValue> {
        let inner_note = self.signed_data(keys).await;
        let mut giftwrap = NostrNote {
            pubkey: keys.get_pubkey().await.unwrap(),
            kind: NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP,
            content: inner_note.to_string(),
            ..Default::default()
        };
        let mut hasher = Sha256::new();
        hasher.update("address".as_bytes());
        let d_tag = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        giftwrap.tags.add_parameter_tag(&d_tag);

        keys.sign_nip44(giftwrap, recipient).await
    }
}
impl Default for ConsumerAddress {
    fn default() -> Self {
        Self {
            lookup: NominatimLookup::default(),
            coordinates: CoordinateStrings::default(),
        }
    }
}
impl ToString for ConsumerAddress {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Into<JsValue> for ConsumerAddress {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

impl From<JsValue> for ConsumerAddress {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}

impl TryFrom<NostrNote> for ConsumerAddress {
    type Error = anyhow::Error;
    fn try_from(value: NostrNote) -> Result<Self, Self::Error> {
        let kind = value.kind;
        if kind != NOSTR_KIND_CONSUMER_PROFILE_ADDRESS {
            return Err(anyhow::anyhow!("Wrong kind"));
        }
        let serde_str = value.content;
        let address: ConsumerAddress = serde_json::from_str(&serde_str)?;
        Ok(address)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsumerAddressIdb {
    nostr_id: String,
    timestamp: i64,
    default: bool,
    note: NostrNote,
    address: ConsumerAddress,
}
impl Into<JsValue> for ConsumerAddressIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for ConsumerAddressIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl TryFrom<NostrNote> for ConsumerAddressIdb {
    type Error = anyhow::Error;
    fn try_from(value: NostrNote) -> Result<Self, Self::Error> {
        let kind = value.kind;
        if kind != NOSTR_KIND_CONSUMER_PROFILE_ADDRESS {
            return Err(anyhow::anyhow!("Wrong kind"));
        }
        let address: ConsumerAddress = serde_json::from_str(&value.content)?;
        let nostr_id = value.id.as_ref().unwrap().to_string();
        let timestamp = value.created_at;
        Ok(ConsumerAddressIdb {
            nostr_id,
            timestamp,
            default: false,
            note: value,
            address,
        })
    }
}

impl ConsumerAddressIdb {
    pub async fn new(address: ConsumerAddress, keys: &UserIdentity) -> Self {
        let note = address.signed_data(keys).await;
        let nostr_id = note.id.as_ref().unwrap().to_string();
        Self {
            nostr_id,
            timestamp: note.created_at,
            default: false,
            note,
            address,
        }
    }
    pub fn is_default(&self) -> bool {
        self.default
    }
    pub fn set_default(&mut self, default: bool) {
        self.default = default;
    }
    pub async fn set_as_default(mut self) -> Result<(), JsValue> {
        self.default = true;
        for address in Self::retrieve_all_from_store().await? {
            if address.id() != self.id() {
                let mut address = address.clone();
                address.set_default(false);
                let _ = address.save_to_store().await;
            }
        }
        self.save_to_store().await?;
        Ok(())
    }
    pub fn signed_note(&self) -> NostrNote {
        self.note.clone()
    }
    pub fn address(&self) -> ConsumerAddress {
        self.address.clone()
    }
    pub fn id(&self) -> String {
        self.nostr_id.clone()
    }
}
impl IdbStoreManager for ConsumerAddressIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            store_name: STORE_NAME_CONSUMER_ADDRESSES,
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            document_key: "nostr_id",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.nostr_id)
    }
}
