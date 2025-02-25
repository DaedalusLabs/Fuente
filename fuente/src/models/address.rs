use nostr_minions::{browser_api::IdbStoreManager, widgets::leaflet::nominatim::NominatimLookup};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::JsValue;

use super::{
    gps::CoordinateStrings,
    nostr_kinds::{NOSTR_KIND_CONSUMER_PROFILE_ADDRESS, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP},
    DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_ADDRESSES,
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
    pub fn signed_data(&self, keys: &NostrKeypair) -> NostrNote {
        let mut new_note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_CONSUMER_PROFILE_ADDRESS,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_event(&mut new_note);
        new_note
    }
    pub fn giftwrapped_data(
        &self,
        keys: &NostrKeypair,
        recipient: String,
    ) -> Result<NostrNote, JsValue> {
        let inner_note = self.signed_data(keys);
        let mut giftwrap = NostrNote {
            pubkey: keys.public_key(),
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

        keys.sign_nip_04_encrypted(&mut giftwrap, recipient)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        Ok(giftwrap)
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
    pub fn new(address: ConsumerAddress, keys: &NostrKeypair) -> Self {
        let note = address.signed_data(keys);
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::init_consumer_db;
    use nostr_minions::browser_api::IdbStoreManager;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn _test_address_idb() -> Result<(), JsValue> {
        init_consumer_db()?;
        let key_1 = NostrKeypair::generate(false);
        let consumer_address = ConsumerAddress::default();
        let address_idb = ConsumerAddressIdb::new(consumer_address.clone(), &key_1);
        address_idb.clone().save_to_store().await.unwrap();

        let key_2 = NostrKeypair::generate(false);
        let address_idb_2 = ConsumerAddressIdb::new(consumer_address, &key_2);
        address_idb_2.clone().save_to_store().await.unwrap();

        let retrieved: ConsumerAddressIdb =
            ConsumerAddressIdb::retrieve_from_store(&address_idb.key())
                .await
                .unwrap();
        assert_eq!(retrieved.id(), address_idb.id());
        assert_eq!(retrieved.address(), address_idb.address());

        let retrieved_2: ConsumerAddressIdb =
            ConsumerAddressIdb::retrieve_from_store(&address_idb_2.key())
                .await
                .unwrap();
        assert_eq!(retrieved_2.id(), address_idb_2.id());
        assert_eq!(retrieved_2.address(), address_idb_2.address());

        let all_addresses = ConsumerAddressIdb::retrieve_all_from_store().await.unwrap();
        assert_eq!(all_addresses.len(), 2);

        let deleted = retrieved.delete_from_store().await;
        let deleted_2 = retrieved_2.delete_from_store().await;
        assert!(deleted.is_ok());
        assert!(deleted_2.is_ok());
        Ok(())
    }
}
