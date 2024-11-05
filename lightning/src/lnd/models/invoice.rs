use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndInvoice {
    r_hash: String,
    payment_request: String,
    add_index: String,
    payment_addr: String,
}
impl TryFrom<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndInvoice {
    pub fn r_hash(&self) -> String {
        self.r_hash.clone()
    }
    pub fn r_hash_url_safe(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let url_safe = BASE64_URL_SAFE.encode(unsafe_str);
        url_safe
    }
    pub fn r_hash_hex(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let hex = hex::encode(unsafe_str);
        hex
    }
    pub fn payment_hash(&self) -> Vec<u8> {
        BASE64_STANDARD.decode(&self.payment_addr).unwrap()
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
    pub fn add_index(&self) -> String {
        self.add_index.clone()
    }
}

