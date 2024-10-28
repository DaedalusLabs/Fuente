use std::io::Read;

use async_channel::{Receiver, Sender};
use base64::prelude::*;

use fuente::models::{
    ln_address::{LnAddressConfirmation, LnAddressPaymentRequest},
    lnd::{
        LndError, LndHodlInvoice, LndHodlInvoiceState, LndInfo, LndInvoice, LndInvoiceRequestBody,
        LndPaymentRequest, LndPaymentResponse, LndResponse,
    },
};
use futures_util::{SinkExt, StreamExt};
use httparse::{Header, Request};
use nostro2::userkeys::UserKeys;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{error, info};

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
        // let mut file = std::fs::File::open(&format!(
        //     "{}/data/chain/bitcoin/regtest/admin.macaroon",
        //     data_dir
        // ))?;
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
        let (user, domain) = ln_url.split_at(ln_url.find('@').unwrap());
        let url = format!("https://{}/.well-known/lnurlp/{}", domain, user);
        let response = self.client.get(&url).send().await?.text().await?;
        let confirmation = LnAddressConfirmation::try_from(response)?;
        let pr_url = format!("{}?amount={}", confirmation.callback, milisats);
        let pay_request_fetch = self.client.get(&pr_url).send().await?.text().await?;
        let pay_request = LnAddressPaymentRequest::try_from(pay_request_fetch)?;
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

pub struct LndWebsocket<R, S> {
    pub receiver: async_channel::Receiver<R>,
    pub sender: async_channel::Sender<S>,
}

impl<R, S> LndWebsocket<R, S>
where
    R: TryFrom<String>
        + TryInto<String>
        + Send
        + Sync
        + 'static
        + Serialize
        + DeserializeOwned
        + Clone,
    <R as TryFrom<std::string::String>>::Error: std::marker::Send + std::fmt::Debug,
    S: TryInto<String> + Send + Sync + 'static,
    <S as TryInto<std::string::String>>::Error: std::marker::Send + std::fmt::Debug,
{
    pub async fn new(
        url: String,
        macaroon: String,
        request: String,
        timeout_pings: usize,
    ) -> anyhow::Result<Self> {
        let random_key = UserKeys::generate().get_public_key();
        let mut headers = [
            Header {
                name: "Grpc-Metadata-macaroon",
                value: macaroon.as_bytes(),
            },
            Header {
                name: "Sec-WebSocket-Key",
                value: random_key.as_bytes(),
            },
            Header {
                name: "Host",
                value: url.as_bytes(),
            },
            Header {
                name: "Connection",
                value: "Upgrade".as_bytes(),
            },
            Header {
                name: "Upgrade",
                value: "websocket".as_bytes(),
            },
            httparse::Header {
                name: "Sec-WebSocket-Version",
                value: "13".as_bytes(),
            },
        ];
        let mut req = Request::new(&mut headers);
        req.method = Some("GET");
        req.path = Some(&request);
        req.version = Some(1);

        // Prepare the websocket connection with SSL
        let danger_conf = Some(tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
            accept_unmasked_frames: true,
            ..Default::default()
        });
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        let (ws, _response) = tokio_tungstenite::connect_async_tls_with_config(
            req,
            danger_conf,
            false,
            Some(tokio_tungstenite::Connector::NativeTls(tls)),
        )
        .await?;
        let (mut websocket_sender, websocket_reader) = ws.split();
        let (tx, receiver) = async_channel::unbounded::<R>();
        let mut boxed_stream = websocket_reader.fuse();
        tokio::spawn(async move {
            let mut pings = 0;
            while let Ok(message) = boxed_stream.select_next_some().await {
                match message {
                    tokio_tungstenite::tungstenite::Message::Text(text) => {
                        if let Ok(response) = LndError::try_from(&text) {
                            error!("{}", response);
                            break;
                        }
                        match LndResponse::try_from(text) {
                            Ok(response) => {
                                if let Err(e) = tx.try_send(response.inner()) {
                                    error!("{}", e);
                                }
                            }
                            Err(e) => {
                                error!("{}", e);
                                break;
                            }
                        }
                    }
                    tokio_tungstenite::tungstenite::Message::Ping(_) => {
                        pings += 1;
                        if pings > timeout_pings {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });
        let (sender, rcv_tx) = async_channel::unbounded::<S>();
        tokio::spawn(async move {
            while let Ok(message) = rcv_tx.recv().await {
                let message: String = message.try_into().unwrap();
                websocket_sender
                    .send(tokio_tungstenite::tungstenite::Message::Text(message))
                    .await
                    .unwrap();
            }
        });
        Ok(Self { receiver, sender })
    }
}

#[cfg(test)]
mod test {

    use fuente::models::lnd::HodlState;
    use tracing::info;
    use tracing_test::traced_test;

    use super::LightningClient;
    #[tokio::test]
    #[traced_test]
    async fn test_connection() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./invoice.macaroon").await?;
        let invoice = client.get_invoice(100).await?;
        info!("{:?}", invoice);
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn test_ln_url() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./invoice.macaroon").await?;
        let ln_address = "42pupusas@blink.sv";
        let pay_request = client
            .get_ln_url_invoice(100000, ln_address.to_string())
            .await?;
        info!("{}", pay_request.pr);
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
            if states.is_closed() {
                tracing::error!("Channel closed");
                break;
            }
        }
        assert!(correct_state);
        Ok(())
    }
}
