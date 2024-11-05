use futures_util::{SinkExt, StreamExt};
use httparse::Header;
use nostro2::userkeys::UserKeys;
use serde::{de::DeserializeOwned, Serialize};

use super::{LndError, LndResponse};

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
        let mut req = httparse::Request::new(&mut headers);
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
        let mut boxed_stream = websocket_reader;
        let (sender, rcv_tx) = async_channel::unbounded::<S>();
        tokio::spawn(async move {
            let mut pings = 0;
            loop {
                tokio::select! {
                    Some(Ok(message)) = boxed_stream.next() => {
                        if let tokio_tungstenite::tungstenite::Message::Text(text) = &message {
                            if let Ok(response) = LndResponse::try_from(text) {
                                if let Err(e) = tx.send(response.inner()).await {
                                    tracing::error!("{}", e);
                                }
                            }
                            if let Ok(response) = LndError::try_from(text) {
                                tracing::error!("{}", response);
                                break;
                            }
                        }
                        if let tokio_tungstenite::tungstenite::Message::Ping(_) = message {
                            pings += 1;
                            if pings > timeout_pings {
                                break;
                            }
                        }
                        if let tokio_tungstenite::tungstenite::Message::Close(e) = message {
                            tracing::error!("Close: {:?}", e);
                            break;
                        }
                    }
                    Ok(message) = rcv_tx.recv() => {
                        let message =  tokio_tungstenite::tungstenite::Message::Text(message.try_into().unwrap());
                        websocket_sender.send(message).await?;
                    }
                    else => {
                        break;
                    }

                }
            }
            websocket_sender.close().await?;
            tx.close();
            Ok::<(), anyhow::Error>(())
        });

        Ok(Self { receiver, sender })
    }
}
#[cfg(test)]
mod test {

    use std::io::Read;

    use crate::LndHodlInvoiceState;
    use tracing::info;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn check_invoice_paid() -> Result<(), anyhow::Error> {
        let url = "lnd.illuminodes.com";
        let client = crate::LightningClient::new(url, "./admin.macaroon").await?;
        let invoice = client.get_invoice(1000).await?;
        tracing::info!("Invoice: {}", invoice);
        let query = format!(
            "wss://{}/v2/invoices/subscribe/{}",
            url,
            invoice.r_hash_url_safe()
        );
        let mut macaroon = vec![];
        let mut file = std::fs::File::open("./admin.macaroon")?;
        file.read_to_end(&mut macaroon)?;
        let lnd_ws = super::LndWebsocket::<LndHodlInvoiceState, String>::new(
            url.to_string(),
            hex::encode(macaroon),
            query,
            1,
        )
        .await?;
        let mut got_state = false;
        loop {
            match lnd_ws.receiver.recv().await {
                Ok(state) => {
                    tracing::info!("State: {}", state);
                    got_state = true;
                    break;
                }
                Err(e) => {
                    info!("Error: {}", e);
                    break;
                }
            }
        }
        assert!(got_state);
        Ok(())
    }
}
