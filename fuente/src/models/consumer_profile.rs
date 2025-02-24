use nostro2::notes::NostrNote;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use sha2::{Digest, Sha256};
use web_sys::wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use nostr_minions::key_manager::UserIdentity;

use nostr_minions::browser_api::IdbStoreManager;

#[cfg(target_arch = "wasm32")]
use super::{NOSTR_KIND_CONSUMER_REGISTRY, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP};

use super::{
    nostr_kinds::NOSTR_KIND_CONSUMER_PROFILE, DB_NAME_FUENTE, DB_VERSION_FUENTE,
    STORE_NAME_CONSUMER_PROFILES,
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
    pub async fn signed_data(&self, keys: &UserIdentity) -> NostrNote {
        let unsigned_note = NostrNote {
            pubkey: keys.get_pubkey().await.expect("no pubkey"),
            kind: NOSTR_KIND_CONSUMER_PROFILE,
            content: self.to_string(),
            ..Default::default()
        };
        keys.sign_nostr_note(unsigned_note)
            .await
            .expect("could not sign")
    }
    pub async fn registry_data(
        &self,
        keys: &UserIdentity,
        recipient: String,
    ) -> Result<NostrNote, JsValue> {
        let inner_note = self.signed_data(keys).await;
        let mut giftwrap = NostrNote {
            pubkey: keys
                .get_pubkey()
                .await
                .ok_or(JsValue::from_str("no pubkey"))?,
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
        keys.sign_nip44(giftwrap, recipient).await
    }
    pub async fn giftwrapped_data(
        &self,
        keys: &UserIdentity,
        recipient: String,
    ) -> Result<NostrNote, JsValue> {
        let inner_note = self.signed_data(keys).await;
        let mut giftwrap = NostrNote {
            pubkey: keys
                .get_pubkey()
                .await
                .ok_or(JsValue::from_str("no pubkey"))?,
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
        keys.sign_nip44(giftwrap, recipient).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsumerProfileIdb {
    pubkey: String,
    note: NostrNote,
    profile: ConsumerProfile,
}

impl ConsumerProfileIdb {
    pub async fn new(profile: ConsumerProfile, keys: &UserIdentity) -> Self {
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
