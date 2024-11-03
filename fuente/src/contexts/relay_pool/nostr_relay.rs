use wasm_bindgen::JsValue;

use crate::{
    browser_api::IdbStoreManager,
    contexts::{init_nostr_db, DB_NAME, DB_VERSION, STORE_NAME_NOSTR_RELAYS},
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UserRelay {
    pub url: String,
    pub read: bool,
    pub write: bool,
}
impl TryFrom<JsValue> for UserRelay {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl Into<JsValue> for UserRelay {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl IdbStoreManager for UserRelay {
    fn config() -> crate::browser_api::IdbStoreConfig {
        crate::browser_api::IdbStoreConfig {
            db_version: DB_VERSION,
            db_name: DB_NAME,
            store_name: STORE_NAME_NOSTR_RELAYS,
            document_key: "url",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.url)
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
    async fn _relay_idb_manager() -> Result<(), JsValue> {
        init_nostr_db()?;
        let user_relay = UserRelay {
            url: "wss://example.com".to_string(),
            read: true,
            write: false,
        };
        user_relay
            .save_to_store()
            .await
            .expect("Error saving to store");
        let retrieved: UserRelay =
            UserRelay::retrieve_from_store(&JsValue::from_str("wss://example.com"))
                .await
                .expect("Error retrieving from store");
        assert_eq!(retrieved.url, "wss://example.com");
        Ok(())
    }
}
