use std::io::Read;

use async_channel::{Receiver, Sender};
use base64::prelude::*;

use crate::{
    ln_address::LnAddressPaymentRequest,
    lnd::{
        LndHodlInvoice, LndHodlInvoiceState, LndInfo, LndInvoice, LndInvoiceRequestBody,
        LndPaymentRequest, LndPaymentResponse,
    },
    LightningAddress, LndWebsocket,
};
use reqwest::header::{HeaderMap, HeaderValue};
use tracing::info;

#[derive(Clone)]
pub struct LightningClient {
    url: &'static str,
    data_dir: &'static str,
    client: reqwest::Client,
}

impl LightningClient {
    pub async fn dud_server() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self {
            url: "localhost:10009",
            client,
            data_dir: "",
        })
    }
    pub async fn new(url: &'static str, data_dir: &'static str) -> anyhow::Result<Self> {
        let mut default_header = HeaderMap::new();
        let macaroon = Self::macaroon(data_dir)?;
        let mut header_value = HeaderValue::from_str(&macaroon).unwrap();
        header_value.set_sensitive(true);
        default_header.insert("Grpc-Metadata-macaroon", header_value);
        default_header.insert("Accept", HeaderValue::from_static("application/json"));
        default_header.insert("Content-Type", HeaderValue::from_static("application/json"));
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(default_header)
            .build()?;
        Ok(Self {
            url,
            client,
            data_dir,
        })
    }
    fn macaroon(data_dir: &'static str) -> anyhow::Result<String> {
        let mut macaroon = vec![];
        let mut file = std::fs::File::open(data_dir)?;
        file.read_to_end(&mut macaroon)?;
        Ok(hex::encode(macaroon))
    }
    pub async fn get_info(&self) -> anyhow::Result<LndInfo> {
        let url = format!("https://{}/v1/getinfo", self.url);
        let response = self.client.get(&url).send().await?;
        let response = response.text().await?;
        LndInfo::try_from(response)
    }
    pub async fn channel_balance(&self) -> anyhow::Result<()> {
        let url = format!("https://{}/v1/balance/channels", self.url);
        let response = self.client.get(&url).send().await?;
        let response = response.text().await?;
        info!("{}", response);
        Ok(())
    }
    pub async fn get_invoice(&self, amount: u64) -> anyhow::Result<LndInvoice> {
        let url = format!("https://{}/v1/invoices", self.url);
        let form = LndInvoiceRequestBody::new(amount.to_string());
        let response = self.client.post(&url).body(form.to_string());
        let response = response.send().await?;
        let response = response.json::<LndInvoice>().await?;
        Ok(response)
    }
    pub async fn get_ln_url_invoice(
        &self,
        milisats: u64,
        ln_url: String,
    ) -> anyhow::Result<LnAddressPaymentRequest> {
        let ln_address = LightningAddress::new(ln_url);
        let pay_request = ln_address.get_invoice(&self.client, milisats).await?;
        Ok(pay_request)
    }
    pub async fn invoice_channel(
        &self,
    ) -> anyhow::Result<(Receiver<LndPaymentResponse>, Sender<LndPaymentRequest>)> {
        let url = format!("wss://{}/v2/router/send?method=POST", self.url);
        let lnd_ws = LndWebsocket::<LndPaymentResponse, LndPaymentRequest>::new(
            self.url.to_string(),
            Self::macaroon(self.data_dir)?,
            url,
            10,
        )
        .await?;
        Ok((lnd_ws.receiver, lnd_ws.sender))
    }
    pub async fn subscribe_to_invoice(
        &self,
        r_hash_url_safe: String,
    ) -> anyhow::Result<async_channel::Receiver<LndHodlInvoiceState>> {
        let query = format!(
            "wss://{}/v2/invoices/subscribe/{}",
            self.url, r_hash_url_safe
        );
        let lnd_ws = LndWebsocket::<LndHodlInvoiceState, String>::new(
            self.url.to_string(),
            Self::macaroon(self.data_dir)?,
            query,
            10,
        )
        .await?;
        return Ok(lnd_ws.receiver.clone());
    }
    pub async fn get_hodl_invoice(
        &self,
        payment_hash: String,
        amount: u64,
    ) -> anyhow::Result<LndHodlInvoice> {
        let url = format!("https://{}/v2/invoices/hodl", self.url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "value": amount, "hash": payment_hash }))
            .send()
            .await?;
        let response = response.text().await?;
        LndHodlInvoice::try_from(response)
    }
    pub async fn settle_htlc(&self, preimage: String) -> anyhow::Result<()> {
        let url = format!("https://{}/v2/invoices/settle", self.url);
        let preimage = hex::decode(preimage)?;
        let preimage = BASE64_URL_SAFE.encode(&preimage);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "preimage": preimage }))
            .send()
            .await?;
        let _test = response.text().await?;
        info!("{}", _test);
        Ok(())
    }
    pub async fn cancel_htlc(&self, payment_hash: String) -> anyhow::Result<()> {
        let url = format!("https://{}/v2/invoices/cancel", self.url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "payment_hash": payment_hash }))
            .send()
            .await?;
        response.text().await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::lnd::HodlState;
    use tracing::info;
    use tracing_test::traced_test;

    use super::LightningClient;
    #[tokio::test]
    #[traced_test]
    async fn test_connection() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let invoice = client.get_invoice(100).await?;
        info!("{:?}", invoice);
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn get_hodl_invoice() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let ln_address = "42pupusas@blink.sv";
        let pay_request = client
            .get_ln_url_invoice(100000, ln_address.to_string())
            .await?;
        let _hodl_invoice = client.get_hodl_invoice(pay_request.r_hash()?, 100).await?;
        let states = client
            .subscribe_to_invoice(pay_request.r_hash_url_safe()?)
            .await?;
        let mut correct_state = false;
        loop {
            info!("Waiting for state");
            match states.recv().await {
                Ok(state) => {
                    info!("{:?}", state.state());
                    match state.state() {
                        HodlState::OPEN => {
                            client.cancel_htlc(pay_request.r_hash_url_safe()?).await?;
                        }
                        HodlState::CANCELED => {
                            correct_state = true;
                            break;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    info!("{}", e);
                    break;
                }
            }
        }
        assert!(correct_state);
        Ok(())
    }

    // #[tokio::test]
    // #[traced_test]
    // async fn settle_htlc() -> Result<(), anyhow::Error> {
    //  use std::sync::Arc;
    //     use tokio::sync::Mutex;
    //     let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
    //     let ln_address = "42pupusas@blink.sv";
    //     let pay_request = client
    //         .get_ln_url_invoice(10000, ln_address.to_string())
    //         .await?;
    //     let hodl_invoice = client.get_hodl_invoice(pay_request.r_hash()?, 20).await?;
    //     info!("{:?}", hodl_invoice.payment_request());
    //     let correct_state = Arc::new(Mutex::new(false));
    //     let states = client
    //         .subscribe_to_invoice(pay_request.r_hash_url_safe()?)
    //         .await?;
    //     let pr = LndPaymentRequest::new(pay_request.pr.clone(), 10, 10.to_string(), false);
    //     let (pay_rx, pay_tx) = client.invoice_channel().await?;
    //     tokio::spawn(async move {
    //         info!("Sent payment request");
    //         loop {
    //             if pay_rx.is_closed() && pay_rx.is_empty() {
    //                 break;
    //             }
    //             info!("Waiting for pr state");
    //             match pay_rx.recv().await {
    //                 Ok(state) => {
    //                     match state.status() {
    //                         InvoicePaymentState::Succeeded => {
    //                             info!("Payment succeeded");
    //                             client.settle_htlc(state.preimage()).await.unwrap();
    //                             info!("Settled");
    //                         }
    //                         _ => {}
    //                     }
    //                 }
    //                 Err(e) => {
    //                     info!("{}", e);
    //                 }
    //             }
    //         }
    //     });
    //     let correct_state_c = correct_state.clone();
    //     loop {
    //         info!("Waiting for state");
    //         match states.recv().await {
    //             Ok(state) => {
    //                 info!("{:?}", state.state());
    //                 match state.state() {
    //                     HodlState::OPEN => {}
    //                     HodlState::ACCEPTED => {
    //                         pay_tx.send(pr.clone()).await.unwrap();
    //                         info!("Sent payment");
    //                     }
    //                     HodlState::SETTLED => {
    //                         info!("REALLY Settled");
    //                         *correct_state_c.lock().await = true;
    //                         break;
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             Err(e) => {
    //                 info!("{}", e);
    //                 break;
    //             }
    //         }
    //     }
    //     assert!(*correct_state.lock().await);
    //     Ok(())
    // }
}
