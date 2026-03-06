use super::config::HubConfig;
use super::errors::HubError;
use super::registry::SpecRegistry;
use super::{ChannelKey, ExchangeId};
use crate::policy::enforce_surface_for_catalog_entry;
use bytes::Bytes;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use ucel_transport::security::{EndpointAllowlist, SubdomainPolicy};

#[derive(Debug, Clone)]
pub struct WsMessage {
    pub raw: Bytes,
}

impl WsMessage {
    pub fn json_value(&self) -> Result<Value, HubError> {
        Ok(serde_json::from_slice(&self.raw)?)
    }
}

pub struct WsHub {
    exchange: ExchangeId,
    config: Arc<HubConfig>,
}

impl WsHub {
    pub(crate) fn new(exchange: ExchangeId, config: Arc<HubConfig>) -> Self {
        Self { exchange, config }
    }

    pub async fn subscribe(
        &self,
        channel_key: impl Into<ChannelKey>,
        params: Option<Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        let key = channel_key.into();
        let spec = SpecRegistry::global()?.resolve_ws(self.exchange, &key)?;
        enforce_surface_for_catalog_entry(self.exchange.as_str(), spec)
            .map_err(|e| HubError::RegistryValidation(e.to_string()))?;
        let url = spec
            .ws_url
            .clone()
            .or_else(|| spec.ws.as_ref().map(|w| w.url.clone()))
            .ok_or_else(|| HubError::RegistryValidation(format!("missing ws url for {key}")))?;

        validate_ws_endpoint(self.exchange, &url)?;

        let (tx, rx) = mpsc::channel(self.config.ws_buffer);
        let channel = spec.channel.clone().unwrap_or_else(|| key.clone());

        tokio::spawn(async move {
            let mut reconnects = 0u8;
            loop {
                match tokio_tungstenite::connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (mut write, mut read) = ws_stream.split();
                        let subscribe =
                            serde_json::json!({"op":"subscribe","channel":channel,"params":params});
                        if write
                            .send(Message::Text(subscribe.to_string()))
                            .await
                            .is_err()
                        {
                            reconnects += 1;
                            if reconnects > 1 {
                                break;
                            }
                            continue;
                        }

                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    if tx
                                        .send(Ok(WsMessage {
                                            raw: Bytes::from(text.into_bytes()),
                                        }))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                }
                                Ok(Message::Binary(bin)) => {
                                    if tx
                                        .send(Ok(WsMessage {
                                            raw: Bytes::from(bin),
                                        }))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                }
                                Ok(Message::Ping(payload)) => {
                                    let _ = write.send(Message::Pong(payload)).await;
                                }
                                Ok(Message::Close(_)) => break,
                                Err(e) => {
                                    let _ = tx.send(Err(HubError::from(e))).await;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(HubError::from(e))).await;
                    }
                }

                reconnects += 1;
                if reconnects > 1 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

fn ws_allowlist(exchange: ExchangeId) -> Result<EndpointAllowlist, HubError> {
    let hosts: Vec<&str> = match exchange {
        ExchangeId::Binance => vec![
            "stream.binance.com",
            "ws-fapi.binance.com",
            "ws-dapi.binance.com",
        ],
        ExchangeId::Bybit => vec!["stream.bybit.com", "stream-testnet.bybit.com"],
        ExchangeId::Coinbase => vec![
            "ws-feed.exchange.coinbase.com",
            "advanced-trade-ws.coinbase.com",
        ],
        ExchangeId::Coincheck => vec!["ws-api.coincheck.com"],
        ExchangeId::Deribit => vec!["www.deribit.com", "test.deribit.com"],
        ExchangeId::Gmocoin => vec!["api.coin.z.com"],
        ExchangeId::Kraken => vec!["ws.kraken.com", "ws-auth.kraken.com"],
        ExchangeId::Okx => vec!["ws.okx.com", "wsaws.okx.com"],
        ExchangeId::Upbit => vec!["api.upbit.com"],
    };
    EndpointAllowlist::new(hosts, SubdomainPolicy::Exact)
        .map_err(|e| HubError::RegistryValidation(e.message))
}

fn validate_ws_endpoint(exchange: ExchangeId, url: &str) -> Result<(), HubError> {
    let al = ws_allowlist(exchange)?;
    al.validate_https_wss(url)
        .map_err(|e| HubError::RegistryValidation(e.message))?;
    Ok(())
}
