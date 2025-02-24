use ::serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsValue;
use nostr_minions::browser_api::IdbStoreManager;
use super::{DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_CONSUMER_FAVORITES};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteStore {
    pub commerce_id: String,
    pub user_id: String,
    pub timestamp: u64,
}

impl FavoriteStore {
    pub fn new(commerce_id: String, user_id: String) -> Self {
        Self {
            commerce_id,
            user_id,
            timestamp: web_sys::js_sys::Date::now() as u64,
        }
    }

    pub fn id(&self) -> String {
        format!("{}-{}", self.user_id, self.commerce_id)
    }
}

impl TryFrom<JsValue> for FavoriteStore {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

impl Into<JsValue> for FavoriteStore {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

impl IdbStoreManager for FavoriteStore {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_CONSUMER_FAVORITES,
            document_key: "commerce_id",
        }
    }

    fn key(&self) -> JsValue {
        JsValue::from_str(&self.id())
    }
}
