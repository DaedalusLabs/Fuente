use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndHodlInvoice {
    payment_addr: String,
    payment_request: String,
    add_index: String,
}
impl LndHodlInvoice {
    pub fn payment_hash(&self) -> Vec<u8> {
        self.payment_addr.as_bytes().to_vec()
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
    pub fn r_hash_url_safe(&self) -> anyhow::Result<String> {
        // let r_hash = self
        //     .payment_request
        //     .parse::<Bolt11Invoice>()
        //     .map_err(|e| anyhow::anyhow!(e.to_string()))?
        //     .p();
        let url_safe = BASE64_URL_SAFE.encode(self.payment_addr.as_bytes());
        Ok(url_safe)
    }
}
impl TryFrom<String> for LndHodlInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndHodlInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndHodlInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HodlState {
    OPEN,
    ACCEPTED,
    CANCELED,
    SETTLED,
}
impl TryFrom<String> for HodlState {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "OPEN" => Ok(HodlState::OPEN),
            "ACCEPTED" => Ok(HodlState::ACCEPTED),
            "CANCELED" => Ok(HodlState::CANCELED),
            "SETTLED" => Ok(HodlState::SETTLED),
            _ => Err(anyhow::anyhow!("Invalid HodlState")),
        }
    }
}
impl TryInto<String> for HodlState {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            HodlState::OPEN => Ok("OPEN".to_string()),
            HodlState::ACCEPTED => Ok("ACCEPTED".to_string()),
            HodlState::CANCELED => Ok("CANCELED".to_string()),
            HodlState::SETTLED => Ok("SETTLED".to_string()),
        }
    }
}

impl Display for HodlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndHodlInvoiceState {
    settled: bool,
    state: HodlState,
    r_hash: String,
    payment_request: String,
}
impl TryFrom<String> for LndHodlInvoiceState {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndHodlInvoiceState {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndHodlInvoiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndHodlInvoiceState {
    pub fn settled(&self) -> bool {
        self.settled
    }
    pub fn state(&self) -> HodlState {
        self.state.clone()
    }
    pub fn r_hash(&self) -> String {
        self.r_hash.clone()
    }
    pub fn r_hash_url_safe(&self) -> String {
        let url_safe = BASE64_URL_SAFE.encode(self.r_hash.as_bytes());
        url_safe
    }
    pub fn payment_request(&self) -> String {
        self.payment_request.clone()
    }
}

