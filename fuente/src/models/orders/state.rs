use bright_lightning::{LnAddressPaymentRequest, LndHodlInvoice};
use nostro2::{keypair::NostrKeypair, notes::NostrNote};
use serde::{Deserialize, Serialize};

use crate::models::{DRIVER_HUB_PUB_KEY, NOSTR_KIND_ORDER_STATE};

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
impl TryFrom<&String> for OrderStatus {
    type Error = anyhow::Error;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl OrderStatus {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Preparing => "Preparing",
            Self::ReadyForDelivery => "Ready for Delivery",
            Self::InDelivery => "In Delivery",
            Self::Completed => "Completed",
            Self::Canceled => "Canceled",
        }
    }
    pub fn theme_color(&self) -> &'static str {
        match self {
            Self::Pending => "bg-gray-100",
            Self::Preparing => "bg-orange-100",
            Self::ReadyForDelivery => "bg-sky-100",
            Self::InDelivery => "bg-orange-100",
            Self::Completed => "bg-green-100",
            Self::Canceled => "bg-red-100",
        }
    }
    pub fn text_color(&self) -> &'static str {
        match self {
            Self::Pending => "text-gray-500",
            Self::Preparing => "text-orange-500",
            Self::ReadyForDelivery => "text-sky-500",
            Self::InDelivery => "text-orange-500",
            Self::Completed => "text-green-500",
            Self::Canceled => "text-red-500",
        }
    }
    pub fn border_color(&self) -> &'static str {
        match self {
            Self::Pending => "border-gray-500",
            Self::Preparing => "border-orange-500",
            Self::ReadyForDelivery => "border-sky-500",
            Self::InDelivery => "border-orange-500",
            Self::Completed => "border-green-500",
            Self::Canceled => "border-red-500",
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
impl TryFrom<String> for OrderPaymentStatus {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s).map_err(|e| anyhow::anyhow!(e))?)
    }
}
impl TryFrom<&String> for OrderPaymentStatus {
    type Error = anyhow::Error;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(s).map_err(|e| anyhow::anyhow!(e))?)
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
impl TryFrom<&str> for OrderParticipant {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "consumer" => Ok(Self::Consumer),
            "commerce" => Ok(Self::Commerce),
            "courier" => Ok(Self::Courier),
            _ => Err(anyhow::anyhow!("Invalid OrderParticipant")),
        }
    }
}
impl Default for OrderParticipant {
    fn default() -> Self {
        Self::Consumer
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
    pub fn signed_order_state(&self, keypair: &NostrKeypair) -> NostrNote {
        let mut new_note = NostrNote {
            kind: NOSTR_KIND_ORDER_STATE,
            content: self.to_string(),
            pubkey: keypair.public_key(),
            ..Default::default()
        };
        keypair.sign_nostr_event(&mut new_note);
        new_note
    }
    pub fn giftwrapped_order(
        &self,
        participant_type: OrderParticipant,
        keypair: &NostrKeypair,
    ) -> anyhow::Result<(NostrNote, NostrNote)> {
        let signed_order = self.signed_order_state(keypair);
        let mut new_note = NostrNote {
            kind: NOSTR_KIND_ORDER_STATE,
            content: signed_order.to_string(),
            pubkey: keypair.public_key(),
            ..Default::default()
        };
        let participant_str: &str = participant_type.into();
        new_note
            .tags
            .add_parameter_tag(&format!("{}-{}", participant_str, self.order_id()));
        new_note.tags.add_custom_tag(
            nostro2::notes::NostrTag::Custom("status"),
            &self.order_status.to_string(),
        );
        new_note.tags.add_custom_tag(
            nostro2::notes::NostrTag::Custom("status"),
            &self.payment_status.to_string(),
        );
        let receiver = match participant_type {
            OrderParticipant::Consumer => self.order.pubkey.clone(),
            OrderParticipant::Commerce => self.get_commerce_pubkey(),
            OrderParticipant::Courier => DRIVER_HUB_PUB_KEY.to_string(),
        };
        keypair.sign_nip_04_encrypted(&mut new_note, receiver)?;
        Ok((signed_order, new_note))
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
    pub fn order_timestamp(&self) -> i64 {
        self.order.created_at
    }
    pub fn locale_date(&self) -> String {
        let timestamp = web_sys::js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
            self.order.created_at as f64 * 1000.0,
        ));
        let locale_options = web_sys::js_sys::Object::new();
        let locale_options = web_sys::js_sys::Intl::DateTimeFormat::new(
            &web_sys::js_sys::Array::of1(&"nl-SR".into()),
            &locale_options,
        );
        let locale_date = timestamp.to_locale_date_string("nl-SR", &locale_options);
        locale_date.into()
    }
    pub fn locale_time(&self) -> String {
        let timestamp = web_sys::js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
            self.order.created_at as f64 * 1000.0,
        ));
        let locale_time = timestamp.to_locale_time_string("nl-SR");
        locale_time.into()
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
        if note.kind != NOSTR_KIND_ORDER_STATE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.content.try_into()?;
        Ok(order)
    }
}
impl TryFrom<&NostrNote> for OrderInvoiceState {
    type Error = anyhow::Error;
    fn try_from(note: &NostrNote) -> Result<Self, Self::Error> {
        if note.kind != NOSTR_KIND_ORDER_STATE {
            return Err(anyhow::anyhow!("Wrong Kind"));
        }
        let order: OrderInvoiceState = note.content.clone().try_into()?;
        Ok(order)
    }
}
