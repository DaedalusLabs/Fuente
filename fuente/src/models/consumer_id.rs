use nostro2::userkeys::UserKeys;
use wasm_bindgen::JsValue;
use web_sys::CryptoKey;

use crate::browser::{
    crypto::{crypto_to_user_keys, user_keys_to_crypto},
    indexed_db::IdbStoreManager,
};

use super::{upgrade_consumer_db, DB_NAME_CONSUMER, DB_VERSION_CONSUMER, STORE_NAME_USER_KEYS};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIdentity {
    id: String,
    crypto_key: CryptoKey,
}

impl UserIdentity {
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub async fn new(keys: UserKeys) -> Self {
        let crypto_key: CryptoKey = user_keys_to_crypto(&keys).await.into();
        let id = keys.get_public_key();
        Self { id, crypto_key }
    }
    pub async fn find_local_identity() -> Result<Self, JsValue>
    where
        Self: IdbStoreManager,
    {
        let crypto_key = Self::retrieve::<CryptoKey>("privateKey")?
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(UserIdentity {
            id: "privateKey".to_string(),
            crypto_key,
        })
    }
    pub async fn new_user_identity() -> Self {
        let user_key = UserKeys::generate_extractable();
        let crypto_key: CryptoKey = user_keys_to_crypto(&user_key).await.into();
        Self::save_value_to_store(crypto_key.clone().into(), "privateKey")
            .expect("Error saving private key")
            .await
            .expect("Error saving private key");
        Self {
            id: "privateKey".to_string(),
            crypto_key,
        }
    }
    pub async fn get_user_keys(&self) -> Result<UserKeys, JsValue> {
        crypto_to_user_keys(self.crypto_key.clone(), true).await
    }
    pub async fn from_new_keys(keys: UserKeys) -> Self {
        let crypto_key: CryptoKey = user_keys_to_crypto(&keys).await.into();
        Self::save_value_to_store(crypto_key.clone().into(), "privateKey")
            .expect("Error saving private key")
            .await
            .expect("Error saving private key");
        Self {
            id: "privateKey".to_string(),
            crypto_key,
        }
    }
}

impl IdbStoreManager for UserIdentity {
    fn store_name() -> &'static str {
        STORE_NAME_USER_KEYS
    }
    fn db_name() -> &'static str {
        DB_NAME_CONSUMER
    }
    fn db_version() -> u32 {
        DB_VERSION_CONSUMER
    }
    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }
    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_consumer_db(event)
    }
}
