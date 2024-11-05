use base64::prelude::*;
use lightning_invoice::Bolt11Invoice;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LnAddressPaymentRequest {
    pub pr: String,
}
impl LnAddressPaymentRequest {
    pub fn r_hash(&self) -> anyhow::Result<String> {
        let r_hash_b = self
            .pr
            .parse::<Bolt11Invoice>()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let r_hash = BASE64_STANDARD.encode(r_hash_b.payment_hash());
        Ok(r_hash)
    }
    pub fn r_hash_url_safe(&self) -> anyhow::Result<String> {
        let r_hash = self
            .pr
            .parse::<Bolt11Invoice>()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let url_safe = BASE64_URL_SAFE.encode(r_hash.payment_hash());
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
pub struct LightningAddress {
    pub ln_url: String,
}
impl LightningAddress {
    pub fn new(ln_url: String) -> Self {
        Self { ln_url }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_confirmation(
        &self,
        client: &reqwest::Client,
    ) -> anyhow::Result<LnAddressConfirmation> {
        let (user, domain) = self.ln_url.split_at(self.ln_url.find('@').unwrap());
        let url = format!("https://{}/.well-known/lnurlp/{}", domain, user);
        let response = client.get(&url).send().await?.text().await?;
        let confirmation = LnAddressConfirmation::try_from(response)?;
        Ok(confirmation)
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_invoice(
        &self,
        client: &reqwest::Client,
        millisatoshis: u64,
    ) -> anyhow::Result<LnAddressPaymentRequest> {
        let confirmation = self.get_confirmation(client).await?;
        if millisatoshis < confirmation.min_sendable {
            return Err(anyhow::anyhow!("Amount too low"));
        }
        let pr_url = format!("{}?amount={}", confirmation.callback, millisatoshis);
        let pay_request_fetch = client.get(&pr_url).send().await?.text().await?;
        let invoice = LnAddressPaymentRequest::try_from(pay_request_fetch)?;
        Ok(invoice)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    #[tracing_test::traced_test]
    pub async fn get_ln_url_invoice() -> Result<(), anyhow::Error> {
        let ln_url = "42pupusas@blink.sv";
        let client = reqwest::Client::new();
        let ln_address = LightningAddress::new(ln_url.to_string());
        let invoice = ln_address.get_invoice(&client, 1000).await?;
        tracing::info!("Invoice: {}", invoice.to_string());
        Ok(())
    }
}
