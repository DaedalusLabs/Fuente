use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::JsValue;

use crate::browser_api::IdbStoreManager;

use super::{
    nostr_kinds::{NOSTR_KIND_CONSUMER_PROFILE, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP},
    upgrade_fuente_db, DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_PROFILES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ConsumerProfile {
    nickname: String,
    telephone: String,
    email: String,
}
impl Default for ConsumerProfile {
    fn default() -> Self {
        Self {
            nickname: "John Doe".to_string(),
            telephone: "11111111".to_string(),
            email: "custom@email.com".to_string(),
        }
    }
}
impl ToString for ConsumerProfile {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl From<JsValue> for ConsumerProfile {
    fn from(value: JsValue) -> Self {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}
impl Into<JsValue> for ConsumerProfile {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<&str> for ConsumerProfile {
    type Error = JsValue;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value).map_err(|e| JsValue::from_str(&e.to_string()))?)
    }
}
impl TryFrom<String> for ConsumerProfile {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<SignedNote> for ConsumerProfile {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: ConsumerProfile = note.get_content().try_into()?;
        Ok(profile)
    }
}
impl ConsumerProfile {
    pub fn new(nickname: String, email: String, telephone: String) -> Self {
        Self {
            nickname,
            telephone,
            email,
        }
    }
    pub fn signed_data(&self, keys: &UserKeys) -> SignedNote {
        let unsigned_note = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_CONSUMER_PROFILE,
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
        hasher.update("profile".as_bytes());
        let d_tag = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        giftwrap.add_tag("d", &d_tag);
        keys.sign_nip_04_encrypted(giftwrap, recipient)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }
    pub fn telephone(&self) -> String {
        self.telephone.clone()
    }
    pub fn email(&self) -> String {
        self.email.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsumerProfileIdb {
    pubkey: String,
    note: SignedNote,
    profile: ConsumerProfile,
}

impl ConsumerProfileIdb {
    pub fn new(profile: ConsumerProfile, keys: &UserKeys) -> Self {
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
    pub fn profile(&self) -> ConsumerProfile {
        self.profile.clone()
    }
    pub fn pubkey(&self) -> String {
        self.pubkey.clone()
    }
}

impl Into<JsValue> for ConsumerProfileIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

impl TryFrom<JsValue> for ConsumerProfileIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

impl TryFrom<SignedNote> for ConsumerProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let pubkey = note.get_pubkey().to_string();
        let profile: ConsumerProfile = note.clone().try_into()?;
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for ConsumerProfileIdb {
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{browser_api::IdbStoreManager, models::init_consumer_db};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn _commerce_profile_idb() -> Result<(), JsValue> {
        init_consumer_db()?;
        let key_1 = UserKeys::generate();
        let consumer_address = ConsumerProfile::default();
        let address_idb = ConsumerProfileIdb::new(consumer_address.clone(), &key_1);
        address_idb.clone().save_to_store().await.unwrap();

        let key_2 = UserKeys::generate();
        let address_idb_2 = ConsumerProfileIdb::new(consumer_address, &key_2);
        address_idb_2.clone().save_to_store().await.unwrap();

        let retrieved: ConsumerProfileIdb =
            ConsumerProfileIdb::retrieve_from_store(&address_idb.key())
                .await
                .unwrap();
        assert_eq!(retrieved.pubkey(), address_idb.pubkey());

        let retrieved_2: ConsumerProfileIdb =
            ConsumerProfileIdb::retrieve_from_store(&address_idb_2.key())
                .await
                .unwrap();
        assert_eq!(retrieved_2.pubkey(), address_idb_2.pubkey());

        let all_addresses = ConsumerProfileIdb::retrieve_all_from_store().await.unwrap();
        assert_eq!(all_addresses.len(), 2);

        let deleted = retrieved.delete_from_store().await;
        let deleted_2 = retrieved_2.delete_from_store().await;
        assert!(deleted.is_ok());
        assert!(deleted_2.is_ok());
        Ok(())
    }
}
