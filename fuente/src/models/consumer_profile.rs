use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::JsValue;

use crate::browser::indexed_db::IdbStoreManager;

use super::{
    nostr_kinds::{NOSTR_KIND_CONSUMER_PROFILE, NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP},
    upgrade_shared_db, DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_CONSUMER_PROFILES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ConsumerProfile {
    nickname: String,
    telephone: String,
    email: String,
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
    id: String,
    note: SignedNote,
    profile: ConsumerProfile,
}

impl ConsumerProfileIdb {
    pub fn new(profile: ConsumerProfile, keys: &UserKeys) -> Self {
        let id = keys.get_public_key().to_string();
        let note = profile.signed_data(keys);
        Self { id, note, profile }
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
    pub fn signed_note(&self) -> SignedNote {
        self.note.clone()
    }
    pub fn profile(&self) -> ConsumerProfile {
        self.profile.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub async fn find_profile(id: &str) -> Result<Self, JsValue> {
        Ok(Self::retrieve::<Self>(id)?
            .await
            .map_err(|e| format!("{:?}", e))?)
    }
    pub async fn find_all_profiles() -> Result<Vec<Self>, JsValue> {
        Self::retrieve_all_from_store::<Self>()?
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl TryInto<JsValue> for ConsumerProfileIdb {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
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
        let id = note.get_pubkey().to_string();
        let profile: ConsumerProfile = note.clone().try_into()?;
        Ok(Self { id, note, profile })
    }
}

impl IdbStoreManager for ConsumerProfileIdb {
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }

    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }

    fn store_name() -> &'static str {
        STORE_NAME_CONSUMER_PROFILES
    }

    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }

    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_shared_db(event)
    }
}
