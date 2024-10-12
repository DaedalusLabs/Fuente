use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::browser::indexed_db::IdbStoreManager;

use super::{upgrade_consumer_db, DB_NAME_SHARED, DB_VERSION_SHARED, STORE_NAME_CONFIGS};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LastSyncTime {
    pub id: String,
    pub timestamp: u64,
}
impl LastSyncTime {
    pub async fn update_sync_time(timestamp: u64) -> Result<(), JsValue> {
        let new = Self {
            id: "last_sync_time".to_string(),
            timestamp,
        };
        new.save_to_store()?
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
    pub async fn get_last_sync_time() -> Result<u64, JsValue> {
        let last_sync_time = Self::retrieve::<Self>("last_sync_time")?
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        Ok(last_sync_time.timestamp)
    }
}
impl TryFrom<JsValue> for LastSyncTime {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl TryInto<JsValue> for LastSyncTime {
    type Error = JsValue;
    fn try_into(self) -> Result<JsValue, Self::Error> {
        Ok(serde_wasm_bindgen::to_value(&self)?)
    }
}
impl IdbStoreManager for LastSyncTime {
    fn store_name() -> &'static str {
        STORE_NAME_CONFIGS
    }
    fn db_name() -> &'static str {
        DB_NAME_SHARED
    }
    fn db_version() -> u32 {
        DB_VERSION_SHARED
    }
    fn document_key(&self) -> JsValue {
        JsValue::from_str(&self.id)
    }
    fn upgrade_db(event: web_sys::Event) -> Result<(), JsValue> {
        upgrade_consumer_db(event)
    }
}
