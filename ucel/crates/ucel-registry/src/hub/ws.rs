use super::config::HubConfig;
use super::errors::HubError;
use super::registry::SpecRegistry;
use super::{ChannelKey, ExchangeId};
use crate::policy::{enforce_private_surface_allowed, enforce_surface_for_catalog_entry};
use bytes::Bytes;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use ucel_core::PrivateWsChannel;
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
        self.subscribe_impl(channel_key.into(), params).await
    }

    pub async fn subscribe_private(
        &self,
        channel: PrivateWsChannel,
        key_id: Option<&str>,
        params: Option<Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        enforce_private_surface_allowed(self.exchange.as_str())
            .map_err(|_| HubError::PrivateWsBlockedByPolicy(self.exchange.as_str().to_string()))?;
        if key_id.is_none() {
            return Err(HubError::MissingPrivateWsAuth(format!("{channel:?}")));
        }
        let channel_key = private_channel_to_catalog_key(channel);
        self.subscribe_impl(channel_key.into(), params).await
    }

    async fn subscribe_impl(
        &self,
        key: ChannelKey,
        params: Option<Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
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

pub fn private_channel_to_catalog_key(channel: PrivateWsChannel) -> &'static str {
    match channel {
        PrivateWsChannel::Balances => "private_balances",
        PrivateWsChannel::Orders => "private_orders",
        PrivateWsChannel::Fills => "private_fills",
        PrivateWsChannel::Positions => "private_positions",
        PrivateWsChannel::Session => "private_session",
    }
}

fn ws_allowlist(exchange: ExchangeId) -> Result<EndpointAllowlist, HubError> {
    let hosts: Vec<&str> = match exchange {
        ExchangeId::Binance => vec!["stream.binance.com"],
        ExchangeId::BinanceUsdm => vec!["fstream.binance.com", "ws-fapi.binance.com"],
        ExchangeId::BinanceCoinm => vec!["dstream.binance.com", "ws-dapi.binance.com"],
        ExchangeId::BinanceOptions => vec!["nbstream.binance.com"],
        ExchangeId::Bitbank => vec!["stream.bitbank.cc"],
        ExchangeId::Bitflyer => vec!["ws.lightstream.bitflyer.com"],
        ExchangeId::Bitget => vec!["ws.bitget.com"],
        ExchangeId::Bithumb => vec!["pubwss.bithumb.com"],
        ExchangeId::Bitmex => vec!["www.bitmex.com"],
        ExchangeId::Bittrade => vec!["ws.bittrade.co.jp"],
        ExchangeId::Bybit => vec!["stream.bybit.com", "stream-testnet.bybit.com"],
        ExchangeId::Coinbase => vec![
            "ws-feed.exchange.coinbase.com",
            "advanced-trade-ws.coinbase.com",
        ],
        ExchangeId::Coincheck => vec!["ws-api.coincheck.com"],
        ExchangeId::Deribit => vec!["www.deribit.com", "test.deribit.com"],
        ExchangeId::Gmocoin => vec!["api.coin.z.com"],
        ExchangeId::Htx => vec!["api.htx.com", "api.huobi.pro"],
        ExchangeId::Kraken => vec!["ws.kraken.com", "ws-auth.kraken.com"],
        ExchangeId::Okx => vec!["ws.okx.com", "wsaws.okx.com"],
        ExchangeId::Sbivc => vec!["stream.sbivc.co.jp"],
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


pub fn public_channel_to_catalog_key(channel: ucel_core::MarketDataChannel) -> &'static str {
    match channel {
        ucel_core::MarketDataChannel::Ticker => "public_ticker",
        ucel_core::MarketDataChannel::Trades => "public_trades",
        ucel_core::MarketDataChannel::OrderBook => "public_orderbook",
        ucel_core::MarketDataChannel::Candles => "public_candles",
    }
}


pub fn ws_ingest_support_summary(exchange: ExchangeId) -> serde_json::Value {
    serde_json::json!({
        "exchange": exchange.as_str(),
        "public": true,
        "private": true,
        "durable_ingest": "enabled_v1"
    })
}
