use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::JsValue;

use nostr_minions::browser_api::IdbStoreManager;

use super::{
    nostr_kinds::{NOSTR_KIND_CONSUMER_PROFILE, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP},
    DB_NAME_FUENTE, DB_VERSION_FUENTE, NOSTR_KIND_CONSUMER_REGISTRY, STORE_NAME_CONSUMER_PROFILES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ConsumerProfile {
    pub nickname: String,
    pub telephone: String,
    pub email: String,
    pub avatar_url: Option<String>,
}
impl Default for ConsumerProfile {
    fn default() -> Self {
        Self {
            nickname: "John Doe".to_string(),
            telephone: "11111111".to_string(),
            email: "custom@email.com".to_string(),
            avatar_url: None,
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
impl TryFrom<NostrNote> for ConsumerProfile {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let profile: ConsumerProfile = note.content.try_into()?;
        Ok(profile)
    }
}
impl ConsumerProfile {
    pub fn new(nickname: String, email: String, telephone: String, avatar: Option<String>) -> Self {
        Self {
            nickname,
            telephone,
            email,
            avatar_url: avatar,
        }
    }
    pub fn signed_data(&self, keys: &NostrKeypair) -> NostrNote {
        let mut unsigned_note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_CONSUMER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_event(&mut unsigned_note);
        unsigned_note
    }
    pub fn registry_data(
        &self,
        keys: &NostrKeypair,
        recipient: String,
    ) -> Result<NostrNote, JsValue> {
        let inner_note = self.signed_data(keys);
        let mut giftwrap = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_CONSUMER_REGISTRY,
            content: inner_note.to_string(),
            ..Default::default()
        };
        let mut hasher = Sha256::new();
        hasher.update("profile".as_bytes());
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
        hasher.update("profile".as_bytes());
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsumerProfileIdb {
    pubkey: String,
    note: NostrNote,
    profile: ConsumerProfile,
}

impl ConsumerProfileIdb {
    pub fn new(profile: ConsumerProfile, keys: &NostrKeypair) -> Self {
        let pubkey = keys.public_key().to_string();
        let note = profile.signed_data(keys);
        Self {
            pubkey,
            note,
            profile,
        }
    }
    pub fn signed_note(&self) -> NostrNote {
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

impl TryFrom<NostrNote> for ConsumerProfileIdb {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_PROFILE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let pubkey = note.pubkey.to_string();
        let profile: ConsumerProfile = note.clone().try_into()?;
        Ok(Self {
            pubkey,
            note,
            profile,
        })
    }
}

impl IdbStoreManager for ConsumerProfileIdb {
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
        let consumer_address = ConsumerProfile::default();
        let address_idb = ConsumerProfileIdb::new(consumer_address.clone(), &key_1);
        address_idb.clone().save_to_store().await.unwrap();

        let key_2 = NostrKeypair::generate(false);
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
