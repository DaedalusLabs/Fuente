use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LndPaymentRequest {
    payment_request: String,  // String
    timeout_seconds: i32,     // Int32
    fee_limit_sat: String,    // Int64
    allow_self_payment: bool, // Bool
}
impl LndPaymentRequest {
    pub fn new(
        payment_request: String,
        timeout_seconds: i32,
        fee_limit_sat: String,
        allow_self_payment: bool,
    ) -> Self {
        Self {
            payment_request,
            timeout_seconds,
            fee_limit_sat,
            allow_self_payment,
        }
    }
}
impl ToString for LndPaymentRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl Into<String> for LndPaymentRequest {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl TryFrom<String> for LndPaymentRequest {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoicePaymentState {
    #[serde(rename = "IN_FLIGHT")]
    InFlight,
    #[serde(rename = "SUCCEEDED")]
    Succeeded,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "INITIATED")]
    Initiaited,
}
impl Display for InvoicePaymentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndPaymentResponse {
    payment_preimage: String,
    status: InvoicePaymentState,
}
impl LndPaymentResponse {
    pub fn preimage(&self) -> String {
        self.payment_preimage.clone()
    }
    pub fn status(&self) -> InvoicePaymentState {
        self.status.clone()
    }
}
impl TryFrom<String> for LndPaymentResponse {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndPaymentResponse {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndPaymentResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
