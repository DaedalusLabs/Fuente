use std::io::Read;

use async_channel::{Receiver, Sender};
use base64::prelude::*;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use httparse::{Header, Request};
use nostro2::userkeys::UserKeys;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use fuente::models::lnd::{
    LndHodlInvoice, LndHodlInvoiceState, LndInfo, LndInvoice, LndInvoiceRequestBody,
    LndPaymentRequest, LndPaymentResponse, LndResponse,
};
use tracing::{error, info};

#[derive(Clone)]
pub struct LndClient {
    url: &'static str,
    data_dir: &'static str,
    client: reqwest::Client,
}

impl LndClient {
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
        let mut file = std::fs::File::open(&format!(
            "{}/data/chain/bitcoin/regtest/admin.macaroon",
            data_dir
        ))?;
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
        invoice: LndInvoice,
    ) -> anyhow::Result<async_channel::Receiver<LndHodlInvoiceState>> {
        let query = format!(
            "wss://{}/v2/invoices/subscribe/{}",
            self.url,
            invoice.r_hash_url_safe()
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
        let (ws, _) = tokio_tungstenite::connect_async_tls_with_config(
            req,
            danger_conf,
            false,
            Some(tokio_tungstenite::Connector::NativeTls(tls)),
        )
        .await?;
        let (mut websocket_sender, websocket_reader) = ws.split();
        let mut stream = websocket_reader.into_stream();
        let (tx, receiver) = async_channel::unbounded::<R>();
        tokio::spawn(async move {
            let mut pings = 0;
            while let Some(message) = stream.next().await {
                if let Ok(message) = message {
                    match message {
                        tokio_tungstenite::tungstenite::Message::Text(text) => {
                            if let Ok(response) = LndResponse::try_from(text) {
                                if let Err(e) = tx.send(response.inner()).await {
                                    error!("{}", e);
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

    use fuente::models::lnd::{HodlState, InvoicePaymentState, LndPaymentRequest};
    use tracing::info;
    use tracing_test::traced_test;

    use super::LndClient;
    #[tokio::test]
    #[traced_test]
    async fn test_simple_payment() -> anyhow::Result<()> {
        //    let server_client = LndClient::new("127.0.0.1:2100", "../test/lnddatadir/").await?;
        //    let user_client = LndClient::new("localhost:4200", "../test/user_lnd_test/").await?;
        //    let (invoice_rx, invoice_tx) = user_client.invoice_channel().await?;
        //    let invoice = server_client.get_invoice(100).await?;
        //    tokio::spawn(async move {
        //        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        //        info!("paying invoice");
        //        invoice_tx
        //            .send(LndPaymentRequest::new(
        //                invoice.payment_request(),
        //                100,
        //                10.to_string(),
        //                false,
        //            ))
        //            .await
        //            .unwrap();
        //        info!("invoice sent");
        //    });
        //    while let Ok(response) = invoice_rx.recv().await {
        //        if response.status() == InvoicePaymentState::Succeeded {
        //            info!("Payment Succeeded");
        //            break;
        //        }
        //    }
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn test_lnd_flow() -> anyhow::Result<()> {
        let server_client = LndClient::new("127.0.0.1:2100", "../test/lnddatadir/").await?;
        let user_client = LndClient::new("localhost:4200", "../test/user_lnd_test/").await?;
        let commerce_client =
            LndClient::new("localhost:4201", "../test/commerce_lnd_test/").await?;
        server_client.channel_balance().await?;
        // We get an invoice from the commerce for the users order
        let invoice_for_order = commerce_client.get_invoice(1000).await?;
        // We build an hodl invoice for it
        let hodl_invoice = server_client
            .get_hodl_invoice(invoice_for_order.r_hash(), 1200)
            .await?;
        // We subscribe to the invoice
        let subscribe = server_client
            .subscribe_to_invoice(invoice_for_order.clone())
            .await?;
        let (_, invoice_tx) = user_client.invoice_channel().await?;
        if let Err(e) = invoice_tx
            .send(LndPaymentRequest::new(
                hodl_invoice.payment_request(),
                120,
                100.to_string(),
                false,
            ))
            .await
        {
            tracing::error!("USER ERROR {}", e);
        };
        info!("invoice sent");
        let (payment_rx, payment_tx) = server_client.invoice_channel().await?;
        while let Ok(response) = subscribe.recv().await {
            match response.state() {
                HodlState::SETTLED => {
                    info!("Hodl settled");
                    break;
                }
                HodlState::CANCELED => {
                    info!("Hodl failed");
                    break;
                }
                HodlState::OPEN => {
                    info!("Hodl open");
                }
                HodlState::ACCEPTED => {
                    info!("Hodl accepted");
                    payment_tx
                        .send(LndPaymentRequest::new(
                            invoice_for_order.payment_request(),
                            100,
                            10.to_string(),
                            false,
                        ))
                        .await
                        .unwrap();
                    while let Ok(response) = payment_rx.recv().await {
                        if response.status() == InvoicePaymentState::Succeeded {
                            info!("Payment Succeeded");
                            server_client.settle_htlc(response.preimage()).await?;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
