use nostr_minions::browser_api::IdbStoreManager;
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::models::{DB_NAME_FUENTE, DB_VERSION_FUENTE, STORE_NAME_ORDER_HISTORY};

use super::state::OrderInvoiceState;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct OrderStateIdb {
    order_id: String,
    timestamp: i64,
    state_note: NostrNote,
}
impl Default for OrderStateIdb {
    fn default() -> Self {
        let order_id = NostrKeypair::generate(false).public_key();
        let order = OrderInvoiceState::default();
        Self {
            order_id,
            timestamp: order.order.created_at,
            state_note: order.order,
        }
    }
}
impl Into<JsValue> for OrderStateIdb {
    fn into(self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}
impl TryFrom<JsValue> for OrderStateIdb {
    type Error = JsValue;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}
impl OrderStateIdb {
    pub fn new(order: NostrNote) -> Result<Self, JsValue> {
        if let Some(order_id) = order.tags.find_first_parameter() {
            Ok(Self {
                order_id: order_id.clone(),
                timestamp: order.created_at,
                state_note: order,
            })
        } else {
            Err(JsValue::from_str("No id tag found"))
        }
    }
    pub async fn find_history(user_keys: &NostrKeypair) -> Result<Vec<OrderInvoiceState>, JsValue> {
        let db_entries = Self::retrieve_all_from_store().await?;
        let order_states = db_entries
            .iter()
            .filter_map(|entry| entry.parse_order(user_keys).ok())
            .collect::<Vec<OrderInvoiceState>>();
        Ok(order_states)
    }
    pub fn signed_note(&self) -> NostrNote {
        self.state_note.clone()
    }
    fn parse_order(&self, user_keys: &NostrKeypair) -> Result<OrderInvoiceState, String> {
        let decrypted = user_keys
            .decrypt_nip_04_content(&self.state_note)
            .map_err(|e| format!("{:?}", e))?;
        OrderInvoiceState::try_from(decrypted).map_err(|e| format!("{:?}", e))
    }
    pub fn id(&self) -> String {
        self.order_id.clone()
    }
}
impl IdbStoreManager for OrderStateIdb {
    fn config() -> nostr_minions::browser_api::IdbStoreConfig {
        nostr_minions::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_ORDER_HISTORY,
            document_key: "order_id",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.order_id)
    }
}
