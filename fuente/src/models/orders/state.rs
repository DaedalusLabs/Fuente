use bright_lightning::{LnAddressPaymentRequest, LndHodlInvoice};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};

use crate::models::{
    DRIVER_HUB_PUB_KEY, NOSTR_KIND_CONSUMER_ORDER_REQUEST, NOSTR_KIND_ORDER_STATE,
    NOSTR_KIND_SERVER_REQUEST, TEST_PUB_KEY,
};

use super::request::OrderRequest;

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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum OrderParticipant {
    Consumer,
    Commerce,
    Courier,
}
impl Into<&str> for OrderParticipant {
    fn into(self) -> &'static str {
        match self {
            Self::Consumer => "consumer",
            Self::Commerce => "commerce",
            Self::Courier => "courier",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct OrderInvoiceState {
    pub order: NostrNote,
    pub commerce_invoice: Option<LnAddressPaymentRequest>,
    pub consumer_invoice: Option<LndHodlInvoice>,
    pub payment_status: OrderPaymentStatus,
    pub order_status: OrderStatus,
    pub courier: Option<NostrNote>,
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
    pub fn sign_update_for(
        &self,
        receiver: OrderParticipant,
        keys: &NostrKeypair,
    ) -> anyhow::Result<NostrNote> {
        let content = self.to_string();
        let order: OrderRequest = self.order.clone().try_into()?;
        let encrypted_to = match receiver {
            OrderParticipant::Consumer => self.order.pubkey.clone(),
            OrderParticipant::Commerce => order.commerce,
            OrderParticipant::Courier => DRIVER_HUB_PUB_KEY.to_string(),
        };

        let mut note = NostrNote {
            pubkey: keys.public_key(),
            kind: NOSTR_KIND_ORDER_STATE,
            content,
            ..Default::default()
        };
        let participant_str: &str = receiver.into();
        note.tags.add_parameter_tag(&format!(
            "{}-{}",
            participant_str,
            self.order.id.as_ref().unwrap()
        ));
        note.tags.add_custom_tag(
            nostro2::notes::NostrTag::Custom("status"),
            self.order_status.to_string().as_str(),
        );
        note.tags.add_custom_tag(
            nostro2::notes::NostrTag::Custom("status"),
            self.payment_status.to_string().as_str(),
        );
        keys.sign_nip_04_encrypted(&mut note, encrypted_to)?;
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
    pub fn get_commerce_pubkey(&self) -> String {
        let order: OrderRequest = self.order.clone().try_into().unwrap();
        order.commerce
    }
    pub fn get_order_request(&self) -> OrderRequest {
        let order: OrderRequest = self.order.clone().try_into().unwrap();
        order
    }
    pub fn order_id(&self) -> String {
        self.order.id.as_ref().unwrap().to_string()
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
