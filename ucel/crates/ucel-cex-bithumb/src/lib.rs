use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ucel_core::{
    Decimal, ErrorCode, OrderBookLevel, OrderBookSnapshot, Side, TickerSnapshot, TradeEvent,
    UcelError,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BithumbRestResponse {
    MarketList(Value),
    TickerList(Vec<TickerItem>),
    OrderBookSnapshot(OrderBookSnapshot),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TickerItem {
    pub market: String,
    pub trade_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub signed_change_rate: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BithumbWsEvent {
    Ticker(TickerSnapshot),
    Trade(TradeEvent),
    Unknown,
}

pub fn map_http_error(status: u16, body: &[u8]) -> UcelError {
    let code = match status {
        429 => ErrorCode::RateLimited,
        500..=599 => ErrorCode::Upstream5xx,
        _ => ErrorCode::Network,
    };
    UcelError::new(
        code,
        format!(
            "bithumb http status={status} body={}",
            String::from_utf8_lossy(body)
        ),
    )
}

pub fn normalize_rest_response(
    endpoint_id: &str,
    body: &[u8],
) -> Result<BithumbRestResponse, UcelError> {
    match endpoint_id {
        "openapi.public.rest.market.list" => {
            let v: Value = serde_json::from_slice(body).map_err(|e| {
                UcelError::new(ErrorCode::Internal, format!("market decode error: {e}"))
            })?;
            Ok(BithumbRestResponse::MarketList(v))
        }
        "openapi.public.rest.ticker.list" => {
            let v: Vec<TickerItem> = serde_json::from_slice(body).map_err(|e| {
                UcelError::new(ErrorCode::Internal, format!("ticker decode error: {e}"))
            })?;
            Ok(BithumbRestResponse::TickerList(v))
        }
        "openapi.public.rest.orderbook.snapshot" => {
            let v: Vec<OrderBookResponse> = serde_json::from_slice(body).map_err(|e| {
                UcelError::new(ErrorCode::Internal, format!("orderbook decode error: {e}"))
            })?;
            let first = v
                .into_iter()
                .next()
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "empty orderbook"))?;
            Ok(BithumbRestResponse::OrderBookSnapshot(OrderBookSnapshot {
                bids: first
                    .orderbook_units
                    .iter()
                    .map(|x| OrderBookLevel {
                        price: x.bid_price,
                        qty: x.bid_size,
                    })
                    .collect(),
                asks: first
                    .orderbook_units
                    .iter()
                    .map(|x| OrderBookLevel {
                        price: x.ask_price,
                        qty: x.ask_size,
                    })
                    .collect(),
                sequence: first.timestamp,
            }))
        }
        _ => Err(UcelError::new(
            ErrorCode::NotSupported,
            "unsupported endpoint id",
        )),
    }
}

pub fn subscribe_frame(channel: &str, symbols: &[&str]) -> Value {
    serde_json::json!({"type": channel, "symbols": symbols})
}

pub fn normalize_ws_event(endpoint_id: &str, raw: &str) -> Result<BithumbWsEvent, UcelError> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("ws decode error: {e}")))?;
    let t = v.get("type").and_then(Value::as_str).unwrap_or_default();
    match (endpoint_id, t) {
        ("openapi.public.ws.trade.snapshot", "trade") => {
            let content = v
                .get("content")
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing content"))?;
            let side = if content.get("buySellGb").and_then(Value::as_str) == Some("1") {
                Side::Buy
            } else {
                Side::Sell
            };
            Ok(BithumbWsEvent::Trade(TradeEvent {
                trade_id: content
                    .get("contNo")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                price: parse_decimal(content, "contPrice")?,
                qty: parse_decimal(content, "contQty")?,
                side,
            }))
        }
        ("openapi.public.ws.ticker.snapshot", "ticker") => {
            let content = v
                .get("content")
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing content"))?;
            Ok(BithumbWsEvent::Ticker(TickerSnapshot {
                bid: parse_decimal(content, "bidPrice")?,
                ask: parse_decimal(content, "askPrice")?,
                last: parse_decimal(content, "closePrice")?,
            }))
        }
        _ => Ok(BithumbWsEvent::Unknown),
    }
}

pub fn heartbeat_response(raw: &str) -> Option<&'static str> {
    if raw.contains("ping") {
        Some("pong")
    } else {
        None
    }
}

fn parse_decimal(v: &Value, key: &str) -> Result<Decimal, UcelError> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| UcelError::new(ErrorCode::Internal, format!("missing {key}")))?
        .parse::<Decimal>()
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("{key} parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct OrderBookResponse {
    timestamp: u64,
    orderbook_units: Vec<OrderBookUnit>,
}

#[derive(Debug, Deserialize)]
struct OrderBookUnit {
    ask_price: Decimal,
    bid_price: Decimal,
    ask_size: Decimal,
    bid_size: Decimal,
}

pub fn normalize_ws_bytes(endpoint_id: &str, body: &Bytes) -> Result<BithumbWsEvent, UcelError> {
    let raw = std::str::from_utf8(body)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("utf8 error: {e}")))?;
    normalize_ws_event(endpoint_id, raw)
}
