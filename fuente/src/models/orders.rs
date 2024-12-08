use bright_lightning::{LnAddressPaymentRequest, LndHodlInvoice};
use nostr_minions::browser_api::IdbStoreManager;
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use wasm_bindgen::JsValue;

use super::{
    address::ConsumerAddress,
    consumer_profile::ConsumerProfile,
    nostr_kinds::{
        NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_ORDER_STATE, NOSTR_KIND_SERVER_REQUEST,
    },
    products::ProductOrder,
    DB_NAME_FUENTE, DB_VERSION_FUENTE, DRIVER_HUB_PUB_KEY, STORE_NAME_ORDER_HISTORY, TEST_PUB_KEY,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct OrderRequest {
    pub commerce: String,
    pub profile: ConsumerProfile,
    pub address: ConsumerAddress,
    pub products: ProductOrder,
}
impl Default for OrderRequest {
    fn default() -> Self {
        Self {
            commerce: "".to_string(),
            profile: ConsumerProfile::default(),
            address: ConsumerAddress::default(),
            products: ProductOrder::default(),
        }
    }
}
impl ToString for OrderRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<NostrNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.content.try_into()?;
        Ok(order)
    }
}
impl TryFrom<&NostrNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: &NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.content.clone().try_into()?;
        Ok(order)
    }
}
impl OrderRequest {
    pub fn new(
        commerce: String,
        profile: ConsumerProfile,
        address: ConsumerAddress,
        products: ProductOrder,
    ) -> Self {
        Self {
            commerce,
            profile,
            address,
            products,
        }
    }
    pub fn sign_request(&self, keys: &NostrKeypair) -> NostrNote {
        let content = self.to_string();
        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_CONSUMER_ORDER_REQUEST,
            content,
            ..Default::default()
        };
        keys.sign_nostr_event(&mut note);
        note
    }
    pub fn giftwrapped_request(
        &self,
        keys: &NostrKeypair,
        recipient: String,
    ) -> anyhow::Result<NostrNote> {
        let note = self.sign_request(keys);
        let content = note.to_string();
        let mut giftwrap = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_SERVER_REQUEST,
            content,
            ..Default::default()
        };
        keys.sign_nip_04_encrypted(&mut giftwrap, recipient)?;
        Ok(giftwrap)
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Preparing,
    ReadyForDelivery,
    InDelivery,
    Completed,
    Canceled,
}
impl ToString for OrderStatus {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for OrderStatus {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl OrderStatus {
    pub fn display(&self) -> String {
        match self {
            Self::Pending => "Pending".to_string(),
            Self::Preparing => "Preparing".to_string(),
            Self::ReadyForDelivery => "Ready for Delivery".to_string(),
            Self::InDelivery => "In Delivery".to_string(),
            Self::Completed => "Completed".to_string(),
            Self::Canceled => "Canceled".to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum OrderPaymentStatus {
    PaymentPending,
    PaymentReceived,
    PaymentFailed,
    PaymentSuccess,
}
impl ToString for OrderPaymentStatus {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct OrderInvoiceState {
    order: NostrNote,
    commerce_invoice: Option<LnAddressPaymentRequest>,
    consumer_invoice: Option<LndHodlInvoice>,
    payment_status: OrderPaymentStatus,
    order_status: OrderStatus,
    courier: Option<NostrNote>,
}
impl Default for OrderInvoiceState {
    fn default() -> Self {
        let order = OrderRequest::default().sign_request(&NostrKeypair::generate(false));
        Self {
            order,
            consumer_invoice: None,
            commerce_invoice: None,
            payment_status: OrderPaymentStatus::PaymentPending,
            order_status: OrderStatus::Pending,
            courier: None,
        }
    }
}
impl ToString for OrderInvoiceState {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<NostrNote> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(note: NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.content.try_into()?;
        Ok(order)
    }
}
impl TryFrom<&NostrNote> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(note: &NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.content.clone().try_into()?;
        Ok(order)
    }
}
impl OrderInvoiceState {
    pub fn new(
        order: NostrNote,
        consumer_invoice: Option<LndHodlInvoice>,
        commerce_invoice: Option<LnAddressPaymentRequest>,
    ) -> Self {
        Self {
            order,
            consumer_invoice,
            commerce_invoice,
            payment_status: OrderPaymentStatus::PaymentPending,
            order_status: OrderStatus::Pending,
            courier: None,
        }
    }
    pub fn update_payment_status(&mut self, status: OrderPaymentStatus) {
        self.payment_status = status;
    }
    pub fn update_order_status(&mut self, status: OrderStatus) {
        self.order_status = status;
    }
    pub fn update_courier(&mut self, courier: NostrNote) {
        self.courier = Some(courier);
    }
    pub fn sign_customer_update(&self, keys: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let content = self.to_string();
        let receiver_pubkey = self.order.pubkey.clone();
        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_ORDER_STATE,
            content,
            ..Default::default()
        };
        note.tags.add_parameter_tag(&format!(
            "{}{}",
            "consumer",
            self.order.id.as_ref().unwrap()
        ));
        keys.sign_nip_04_encrypted(&mut note, receiver_pubkey)?;
        Ok(note)
    }
    pub fn sign_commerce_update(&self, keys: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let content = self.to_string();
        let order: OrderRequest = self.order.clone().try_into()?;
        let commerce = order.commerce;

        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_ORDER_STATE,
            content,
            ..Default::default()
        };
        note.tags.add_parameter_tag(&format!(
            "{}{}",
            "business",
            self.order.id.as_ref().unwrap()
        ));
        keys.sign_nip_04_encrypted(&mut note, commerce)?;
        Ok(note)
    }
    pub fn sign_courier_update(&self, keys: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let content = self.to_string();
        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_ORDER_STATE,
            content,
            ..Default::default()
        };
        note.tags
            .add_parameter_tag(&format!("{}{}", "courier", self.order.id.as_ref().unwrap()));
        keys.sign_nip_04_encrypted(&mut note, DRIVER_HUB_PUB_KEY.to_string())?;
        Ok(note)
    }
    pub fn sign_server_request(&self, keys: &NostrKeypair) -> anyhow::Result<NostrNote> {
        let content = self.to_string();
        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_ORDER_STATE,
            content,
            ..Default::default()
        };
        note.tags
            .add_parameter_tag(&self.order.id.as_ref().unwrap());
        keys.sign_nostr_event(&mut note);

        let mut giftwrap = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_SERVER_REQUEST,
            content: note.to_string(),
            ..Default::default()
        };
        keys.sign_nip_04_encrypted(&mut giftwrap, TEST_PUB_KEY.to_string())?;
        Ok(giftwrap)
    }
    pub fn get_order(&self) -> NostrNote {
        self.order.clone()
    }
    pub fn get_payment_status(&self) -> OrderPaymentStatus {
        self.payment_status.clone()
    }
    pub fn get_order_status(&self) -> OrderStatus {
        self.order_status.clone()
    }
    pub fn get_commerce_invoice(&self) -> Option<LnAddressPaymentRequest> {
        self.commerce_invoice.clone()
    }
    pub fn get_consumer_invoice(&self) -> Option<LndHodlInvoice> {
        self.consumer_invoice.clone()
    }
    pub fn get_courier(&self) -> Option<NostrNote> {
        self.courier.clone()
    }
    pub fn get_order_request(&self) -> OrderRequest {
        let order: OrderRequest = self.order.clone().try_into().unwrap();
        order
    }
    pub fn id(&self) -> String {
        self.order.id.as_ref().unwrap().to_string()
    }
}

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
            state_note: order.get_order(),
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
        if let Some(order_id) = order
            .tags
            .find_first_parameter()
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::init_consumer_db;
    use nostr_minions::browser_api::IdbStoreManager;

    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn _commerce_profile_idb() -> Result<(), JsValue> {
        init_consumer_db()?;
        let order_idb = OrderStateIdb::default();
        order_idb.clone().save_to_store().await?;

        let db_entries = OrderStateIdb::retrieve_all_from_store().await?;
        assert_eq!(db_entries.len(), 1);

        let order_idb_2 = OrderStateIdb::default();
        order_idb_2.clone().save_to_store().await?;

        let db_entries = OrderStateIdb::retrieve_all_from_store().await?;
        assert_eq!(db_entries.len(), 2);
        order_idb.delete_from_store().await?;
        order_idb_2.delete_from_store().await?;
        assert_eq!(OrderStateIdb::retrieve_all_from_store().await?.len(), 0);

        Ok(())
    }
}
