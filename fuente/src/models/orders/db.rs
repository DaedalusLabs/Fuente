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
        let order_state = OrderInvoiceState::try_from(&order)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        Ok(Self {
            order_id: order_state.order_id(),
            timestamp: order.created_at,
            state_note: order,
        })
    }
    pub async fn find_history() -> Result<Vec<OrderInvoiceState>, JsValue> {
        let db_entries = Self::retrieve_all_from_store().await?;
        let order_states = db_entries
            .iter()
            .filter_map(|entry| entry.parse_order().ok())
            .collect::<Vec<OrderInvoiceState>>();
        Ok(order_states)
    }
    pub async fn save(&self) -> Result<(), JsValue> {
        self.clone().save_to_store().await
    }
    pub fn signed_note(&self) -> NostrNote {
        self.state_note.clone()
    }
    fn parse_order(&self) -> Result<OrderInvoiceState, String> {
        OrderInvoiceState::try_from(&self.state_note).map_err(|e| format!("{:?}", e))
    }
    pub fn id(&self) -> String {
        self.order_id.clone()
    }
    pub async fn last_saved_timestamp() -> anyhow::Result<i64> {
        let db_entries = Self::retrieve_all_from_store()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let last_entry = db_entries
            .iter()
            .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
            .ok_or_else(|| anyhow::anyhow!("No entries found"))?;
        Ok(last_entry.timestamp)
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
