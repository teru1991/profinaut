use crate::hub::{ExchangeId, Hub, HubError, WsMessage};
use futures_util::Stream;
use serde_json::Value;
use std::pin::Pin;

#[derive(Clone)]
pub struct MarketDataFacade {
    hub: Hub,
    exchange: ExchangeId,
}

impl MarketDataFacade {
    pub fn new(hub: Hub, exchange: ExchangeId) -> Self {
        Self { hub, exchange }
    }

    pub async fn get_ticker(&self, symbol: &str) -> Result<Value, HubError> {
        self.call_public_rest("public_ticker", symbol).await
    }

    pub async fn get_trades(&self, symbol: &str) -> Result<Value, HubError> {
        self.call_public_rest("public_trades", symbol).await
    }

    pub async fn get_orderbook_snapshot(&self, symbol: &str) -> Result<Value, HubError> {
        self.call_public_rest("public_orderbook", symbol).await
    }

    pub async fn get_candles(&self, symbol: &str) -> Result<Value, HubError> {
        self.call_public_rest("public_candles", symbol).await
    }

    pub async fn list_symbols(&self) -> Result<Value, HubError> {
        let resp = self
            .hub
            .rest(self.exchange)
            .call("public_symbols", None, None)
            .await?;
        resp.json_value()
    }

    pub async fn get_market_meta(&self, symbol: &str) -> Result<Value, HubError> {
        self.call_public_rest("public_market_meta", symbol).await
    }

    pub async fn subscribe_ticker(
        &self,
        symbol: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.subscribe("public_ticker", symbol).await
    }

    pub async fn subscribe_trades(
        &self,
        symbol: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.subscribe("public_trades", symbol).await
    }

    pub async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.subscribe("public_orderbook", symbol).await
    }

    pub async fn subscribe_candles(
        &self,
        symbol: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.subscribe("public_candles", symbol).await
    }

    pub fn preview_market_data_plan(&self, symbol: &str) -> Value {
        serde_json::json!({
            "exchange": self.exchange.as_str(),
            "symbol": symbol,
            "rest": ["public_ticker", "public_trades", "public_orderbook", "public_candles", "public_symbols", "public_market_meta"],
            "ws": ["public_ticker", "public_trades", "public_orderbook", "public_candles"]
        })
    }

    pub fn preview_ingest_plan(&self, symbol: &str, channels: &[&str]) -> Value {
        serde_json::json!({
            "exchange": self.exchange.as_str(),
            "symbol": symbol,
            "channels": channels,
            "lifecycle": ["Planned", "PendingConnect", "Connecting", "AwaitingAck", "Active"]
        })
    }

    pub fn start_ingest(&self) -> String {
        format!("ws-ingest-started:{}", self.exchange.as_str())
    }

    pub fn stop_ingest(&self) -> String {
        format!("ws-ingest-stopped:{}", self.exchange.as_str())
    }

    pub fn drain_ingest(&self) -> String {
        format!("ws-ingest-drained:{}", self.exchange.as_str())
    }

    async fn call_public_rest(&self, op: &'static str, symbol: &str) -> Result<Value, HubError> {
        let resp = self
            .hub
            .rest(self.exchange)
            .call(op, Some(&[("symbol", symbol)]), None)
            .await?;
        resp.json_value()
    }

    async fn subscribe(
        &self,
        channel: &'static str,
        symbol: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.hub
            .ws(self.exchange)
            .subscribe(channel, Some(serde_json::json!({"symbol": symbol})))
            .await
    }
}
