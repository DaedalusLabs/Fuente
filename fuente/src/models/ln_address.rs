use lightning_invoice::Bolt11Invoice;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnAddressPaymentRequest {
    pub pr: String,
}
impl LnAddressPaymentRequest {
    pub fn r_hash(&self) -> anyhow::Result<String> {
        let r_hash_b = self
            .pr
            .parse::<Bolt11Invoice>()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?.signable_hash();
        let r_hash = BASE64_STANDARD.encode(r_hash_b);
        Ok(r_hash)
    }
    pub fn r_hash_url_safe(&self) -> anyhow::Result<String> {
        let r_hash = self
            .pr
            .parse::<Bolt11Invoice>()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?.signable_hash();
        let url_safe = BASE64_URL_SAFE.encode(r_hash);
        Ok(url_safe)
    }
}
impl ToString for LnAddressPaymentRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for LnAddressPaymentRequest {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnAddressConfirmation {
    pub callback: String,
    #[serde(rename = "minSendable")]
    pub min_sendable: u64,
    #[serde(rename = "maxSendable")]
    pub max_sendable: u64,
}
impl ToString for LnAddressConfirmation {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl TryFrom<String> for LnAddressConfirmation {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
