use crate::browser_api::IdbStoreManager;
use lightning::{LnAddressPaymentRequest, LndHodlInvoice};
use nostro2::{
    notes::{Note, SignedNote},
    userkeys::UserKeys,
};
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
    upgrade_fuente_db, DB_NAME_FUENTE, DB_VERSION_FUENTE, DRIVER_HUB_PUB_KEY,
    STORE_NAME_ORDER_HISTORY, TEST_PUB_KEY,
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
impl TryFrom<SignedNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.get_content().try_into()?;
        Ok(order)
    }
}
impl TryFrom<&SignedNote> for OrderRequest {
    type Error = anyhow::Error;
    fn try_from(note: &SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderRequest = note.get_content().try_into()?;
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
    pub fn sign_request(&self, keys: &UserKeys) -> SignedNote {
        let content = self.to_string();
        let note = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_CONSUMER_ORDER_REQUEST,
            &content,
        );
        keys.sign_nostr_event(note)
    }
    pub fn giftwrapped_request(
        &self,
        keys: &UserKeys,
        recipient: String,
    ) -> anyhow::Result<SignedNote> {
        let note = self.sign_request(keys);
        let content = note.to_string();
        let note = Note::new(&keys.get_public_key(), NOSTR_KIND_SERVER_REQUEST, &content);
        keys.sign_nip_04_encrypted(note, recipient)
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
    order: SignedNote,
    commerce_invoice: Option<LnAddressPaymentRequest>,
    consumer_invoice: Option<LndHodlInvoice>,
    payment_status: OrderPaymentStatus,
    order_status: OrderStatus,
    courier: Option<SignedNote>,
}
impl Default for OrderInvoiceState {
    fn default() -> Self {
        let order = OrderRequest::default().sign_request(&UserKeys::generate());
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
impl TryFrom<SignedNote> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(note: SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.get_content().try_into()?;
        Ok(order)
    }
}
impl TryFrom<&SignedNote> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(note: &SignedNote) -> Result<Self, Self::Error> {
        if note.get_kind() != NOSTR_KIND_CONSUMER_ORDER_REQUEST {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.get_content().try_into()?;
        Ok(order)
    }
}
impl OrderInvoiceState {
    pub fn new(
        order: SignedNote,
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
    pub fn update_courier(&mut self, courier: SignedNote) {
        self.courier = Some(courier);
    }
    pub fn sign_customer_update(&self, keys: &UserKeys) -> anyhow::Result<SignedNote> {
        let content = self.to_string();
        let pubkey = self.order.get_pubkey();
        let mut note = Note::new(&keys.get_public_key(), NOSTR_KIND_ORDER_STATE, &content);
        note.add_tag("d", &format!("{}{}", "consumer", self.order.get_id()));
        Ok(keys.sign_nip_04_encrypted(note, pubkey.to_string())?)
    }
    pub fn sign_commerce_update(&self, keys: &UserKeys) -> anyhow::Result<SignedNote> {
        let content = self.to_string();
        let order: OrderRequest = self.order.clone().try_into()?;
        let commerce = order.commerce;
        let mut note = Note::new(&keys.get_public_key(), NOSTR_KIND_ORDER_STATE, &content);
        note.add_tag("d", &format!("{}{}", "business", self.order.get_id()));
        Ok(keys.sign_nip_04_encrypted(note, commerce)?)
    }
    pub fn sign_courier_update(&self, keys: &UserKeys) -> anyhow::Result<SignedNote> {
        let content = self.to_string();
        let mut note = Note::new(&keys.get_public_key(), NOSTR_KIND_ORDER_STATE, &content);
        note.add_tag("d", &format!("{}{}", "driver", self.order.get_id()));
        Ok(keys.sign_nip_04_encrypted(note, DRIVER_HUB_PUB_KEY.to_string())?)
    }
    pub fn sign_server_request(&self, keys: &UserKeys) -> anyhow::Result<SignedNote> {
        let content = self.to_string();
        let mut note = Note::new(&keys.get_public_key(), NOSTR_KIND_ORDER_STATE, &content);
        note.add_tag("d", &self.order.get_id());
        let signed_request = keys.sign_nostr_event(note);
        let giftwrap = Note::new(
            &keys.get_public_key(),
            NOSTR_KIND_SERVER_REQUEST,
            &signed_request.to_string(),
        );
        Ok(keys.sign_nip_04_encrypted(giftwrap, TEST_PUB_KEY.to_string())?)
    }
    pub fn get_order(&self) -> SignedNote {
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
    pub fn get_courier(&self) -> Option<SignedNote> {
        self.courier.clone()
    }
    pub fn get_order_request(&self) -> OrderRequest {
        let order: OrderRequest = self.order.clone().try_into().unwrap();
        order
    }
    pub fn id(&self) -> String {
        self.order.get_id()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct OrderStateIdb {
    order_id: String,
    timestamp: u64,
    state_note: SignedNote,
}
impl Default for OrderStateIdb {
    fn default() -> Self {
        let order_id = UserKeys::generate().get_public_key();
        let order = OrderInvoiceState::default();
        Self {
            order_id,
            timestamp: order.order.get_created_at(),
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
    pub fn new(order: SignedNote) -> Result<Self, JsValue> {
        if let Some(d_tags) = order.get_tags_by_id("d") {
            let order_id = d_tags[0].clone();
            Ok(Self {
                order_id,
                timestamp: order.get_created_at(),
                state_note: order,
            })
        } else {
            Err(JsValue::from_str("No id tag found"))
        }
    }
    pub async fn find_history(user_keys: &UserKeys) -> Result<Vec<OrderInvoiceState>, JsValue> {
        let db_entries = Self::retrieve_all_from_store().await?;
        let order_states = db_entries
            .iter()
            .filter_map(|entry| entry.parse_order(user_keys).ok())
            .collect::<Vec<OrderInvoiceState>>();
        Ok(order_states)
    }
    pub fn signed_note(&self) -> SignedNote {
        self.state_note.clone()
    }
    fn parse_order(&self, user_keys: &UserKeys) -> Result<OrderInvoiceState, String> {
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
    fn config() -> crate::browser_api::IdbStoreConfig {
        crate::browser_api::IdbStoreConfig {
            db_name: DB_NAME_FUENTE,
            db_version: DB_VERSION_FUENTE,
            store_name: STORE_NAME_ORDER_HISTORY,
            document_key: "order_id",
        }
    }
    fn key(&self) -> JsValue {
        JsValue::from_str(&self.order_id)
    }
    fn upgrade_db(db: web_sys::IdbDatabase) -> Result<(), JsValue> {
        upgrade_fuente_db(db)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{browser_api::IdbStoreManager, models::init_consumer_db};
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
