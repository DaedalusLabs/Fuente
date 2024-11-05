use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::browser_api::IdbStoreManager;

use super::{DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_CONFIGS};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LastSyncTime {
    pub tag: String,
    pub timestamp: u64,
}
impl LastSyncTime {
    pub async fn update_sync_time(timestamp: u64) -> Result<(), JsValue> {
        let new = Self {
            tag: "last_sync_time".to_string(),
            timestamp,
        };
        new.save_to_store().await
    }
    pub async fn get_last_sync_time() -> Result<u64, JsValue> {
        let last_sync_time: Self =
            Self::retrieve_from_store(&JsValue::from_str("last_sync_time")).await?;
        Ok(last_sync_time.timestamp)
    }
}
impl TryFrom<JsValue> for LastSyncTime {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl Into<JsValue> for LastSyncTime {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl IdbStoreManager for LastSyncTime {
    fn config() -> crate::browser_api::IdbStoreConfig {
        crate::browser_api::IdbStoreConfig {
            db_name: DB_NAME_SHARED,
            db_version: DB_VERSION_SHARED,
            store_name: STORE_NAME_CONFIGS,
            document_key: "tag",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.tag)
    }
}
