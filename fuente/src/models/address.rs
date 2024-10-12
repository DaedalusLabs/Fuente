use crate::browser::{indexed_db::IdbStoreManager, nominatim::NominatimLookup};
use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::JsValue;

use super::{
    gps::CoordinateStrings,
    nostr_kinds::{NOSTR_KIND_CONSUMER_PROFILE_ADDRESS, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP},
    upgrade_shared_db, DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_CONSUMER_ADDRESSES,
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
    pub fn signed_data(&self, keys: &UserKeys) -> SignedNote {
        let unsigned_note = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_CONSUMER_PROFILE_ADDRESS,
            &self.to_string(),
        );
        keys.sign_nostr_event(unsigned_note)
    }
    pub fn giftwrapped_data(
        &self,
        keys: &UserKeys,
        recipient: String,
    ) -> Result<SignedNote, JsValue> {
        let inner_note = self.signed_data(keys);
        let mut giftwrap = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP,
            &inner_note.to_string(),
        );
        let mut hasher = Sha256::new();
        hasher.update("address".as_bytes());
        let d_tag = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        giftwrap.add_tag("d", &d_tag);
        keys.sign_nip_04_encrypted(giftwrap, recipient)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
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

impl TryFrom<SignedNote> for ConsumerAddress {
    type Error = anyhow::Error;
    fn try_from(value: SignedNote) -> Result<Self, Self::Error> {
        let kind = value.get_kind();
        if kind != NOSTR_KIND_CONSUMER_PROFILE_ADDRESS {
            return Err(anyhow::anyhow!("Wrong kind"));
        }
        let serde_str = value.get_content();
        let address: ConsumerAddress = serde_json::from_str(&serde_str)?;
        Ok(address)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsumerAddressIdb {
    id: String,
    timestamp: u64,
    default: bool,
    note: SignedNote,
    address: ConsumerAddress,
}

impl TryInto<JsValue> for ConsumerAddressIdb {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}
impl TryFrom<JsValue> for ConsumerAddressIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl TryFrom<SignedNote> for ConsumerAddressIdb {
    type Error = anyhow::Error;
    fn try_from(value: SignedNote) -> Result<Self, Self::Error> {
        let kind = value.get_kind();
        if kind != NOSTR_KIND_CONSUMER_PROFILE_ADDRESS {
            return Err(anyhow::anyhow!("Wrong kind"));
        }
        let serde_str = value.get_content();
        let address: ConsumerAddress = serde_json::from_str(&serde_str)?;
        let id = value.get_id().to_string();
        let timestamp = value.get_created_at();
        Ok(ConsumerAddressIdb {
            id,
            timestamp,
            default: false,
            note: value,
            address,
        })
    }
}

impl ConsumerAddressIdb {
    pub fn new(address: ConsumerAddress, keys: &UserKeys) -> Self {
        let note = address.signed_data(keys);
        let id = note.get_id().to_string();
        Self {
            id,
            timestamp: note.get_created_at(),
            default: false,
            note,
            address,
        }
    }
    pub async fn save(self) -> Result<(), JsValue> {
        self.save_to_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub async fn delete(self) -> Result<(), JsValue> {
        self.delete_from_store()?
            .await
            .map_err(|e| format!("{:?}", e).into())
    }
    pub fn is_default(&self) -> bool {
        self.default
    }
    pub fn set_default(&mut self, default: bool) {
        self.default = default;
    }
    pub async fn set_as_default(mut self) -> Result<(), JsValue> {
        self.default = true;
        for address in Self::find_all().await? {
            if address.id() != self.id() {
                let mut address = address.clone();
                address.set_default(false);
                let _ = address.save().await;
            }
        }
        self.save().await?;
        Ok(())
    }
    pub async fn find_all() -> Result<Vec<Self>, JsValue> {
        Self::retrieve_all_from_store::<Self>()?
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn signed_note(&self) -> SignedNote {
        self.note.clone()
    }
    pub fn address(&self) -> ConsumerAddress {
        self.address.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
}
impl IdbStoreManager for ConsumerAddressIdb {
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }

    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }

    fn store_name() -> &'static str {
        STORE_NAME_CONSUMER_ADDRESSES
    }

    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }

    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_shared_db(event)
    }
}
