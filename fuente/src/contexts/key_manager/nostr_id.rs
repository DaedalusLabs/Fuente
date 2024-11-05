use nostro2::userkeys::UserKeys;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CryptoKey;

use crate::{
    browser_api::{BrowserCrypto, IdbStoreConfig, IdbStoreManager},
    contexts::{init_nostr_db, DB_NAME, DB_VERSION, STORE_NAME_NOSTR_IDS},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIdentity {
    pubkey: String,
    crypto_key: CryptoKey,
}

impl UserIdentity {
    pub async fn find_local_identity() -> Result<Self, JsValue>
    where
        Self: IdbStoreManager,
    {
        Self::retrieve_from_store::<Self>(&JsValue::from_str("privateKey")).await
    }
    pub async fn new_local_identity() -> Result<Self, JsValue> {
        let user_key = UserKeys::generate_extractable();
        let crypto_key = BrowserCrypto::default()
            .crypto_key_from_bytes(&user_key.get_secret_key())
            .await?;
        let new_identity = UserIdentity {
            pubkey: "privateKey".to_string(),
            crypto_key,
        };
        new_identity.clone().save_to_store().await?;
        Ok(new_identity)
    }
    pub async fn from_new_keys(keys: UserKeys) -> Result<Self, JsValue> {
        let crypto_key: CryptoKey = BrowserCrypto::default()
            .crypto_key_from_bytes(&keys.get_secret_key())
            .await?;
        let user_identity = UserIdentity {
            pubkey: "privateKey".to_string(),
            crypto_key,
        };
        user_identity.clone().save_to_store().await?;
        Ok(user_identity)
    }
    pub async fn get_user_keys(&self) -> Result<UserKeys, JsValue> {
        let key = BrowserCrypto::default()
            .crypto_key_to_hex(self.crypto_key.clone())
            .await?;
        Ok(UserKeys::new(&key).map_err(|e| JsValue::from_str(&e.to_string()))?)
    }
    pub fn get_pubkey(&self) -> String {
        self.pubkey.clone()
    }
}
impl Into<JsValue> for UserIdentity {
    fn into(self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("pubkey"),
            &JsValue::from_str(&self.pubkey),
        )
        .unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("crypto_key"),
            &self.crypto_key.into(),
        )
        .unwrap();
        obj.into()
    }
}
impl TryFrom<JsValue> for UserIdentity {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let obj = js_sys::Object::try_from(&value).ok_or(JsValue::from_str("Not an object"))?;
        let pubkey = js_sys::Reflect::get(&obj, &JsValue::from_str("pubkey"))?
            .as_string()
            .ok_or(JsValue::from_str("id not found"))?;
        let crypto_key = js_sys::Reflect::get(&obj, &JsValue::from_str("crypto_key"))?;
        let crypto_key = crypto_key.dyn_into::<CryptoKey>()?;
        Ok(UserIdentity { pubkey, crypto_key })
    }
}
impl IdbStoreManager for UserIdentity {
    fn config() -> IdbStoreConfig {
        IdbStoreConfig {
            db_version: DB_VERSION,
            db_name: DB_NAME,
            store_name: STORE_NAME_NOSTR_IDS,
            document_key: "pubkey",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.pubkey)
    }
    fn upgrade_db(_db: web_sys::IdbDatabase) -> Result<(), JsValue> {
        init_nostr_db()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen_test]
    async fn _user_identity_db() -> Result<(), JsValue> {
        init_nostr_db()?;
        let user_identity = UserIdentity::new_local_identity().await.unwrap();
        let user_keys = user_identity.get_user_keys().await.unwrap();
        let user_identity = UserIdentity::find_local_identity().await.unwrap();
        let user_keys2 = user_identity.get_user_keys().await.unwrap();
        assert_eq!(user_keys, user_keys2);
        Ok(())
    }
}
