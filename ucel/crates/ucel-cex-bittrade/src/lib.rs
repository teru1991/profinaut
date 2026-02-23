use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use ucel_core::{
    ErrorCode, OpName, OrderBookDelta, OrderBookLevel, OrderBookSnapshot, TradeEvent, UcelError,
};
use ucel_transport::{enforce_auth_boundary, RequestContext, Transport, WsConnectRequest};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    ws_channels: Vec<WsSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsSpec {
    pub id: String,
    pub access: String,
    pub ws_url: String,
    pub channel: String,
}

pub fn ws_specs() -> Vec<WsSpec> {
    serde_json::from_str::<Catalog>(include_str!(
        "../../../../docs/exchanges/bittrade/catalog.json"
    ))
    .expect("bittrade catalog")
    .ws_channels
}

#[derive(Debug, Default)]
pub struct WsCounters {
    pub ws_backpressure_drops_total: AtomicU64,
    pub ws_reconnect_total: AtomicU64,
    pub ws_resubscribe_total: AtomicU64,
}

pub struct BittradeWsBackpressure {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
    counters: Arc<WsCounters>,
}
impl BittradeWsBackpressure {
    pub fn new(cap: usize, counters: Arc<WsCounters>) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx, counters }
    }
    pub fn try_enqueue(&self, msg: Bytes) {
        if self.tx.try_send(msg).is_err() {
            self.counters
                .ws_backpressure_drops_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }
    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketEvent {
    Kline {
        ch: String,
        close: f64,
    },
    OrderBookDelta(OrderBookDelta),
    Bbo {
        ch: String,
        bid: f64,
        ask: f64,
    },
    Ticker {
        ch: String,
        close: f64,
    },
    Trade(TradeEvent),
    AccountUpdate {
        ch: String,
        count: usize,
    },
    TradeClearing {
        ch: String,
        fields: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsSubscription {
    pub channel_id: String,
    pub symbol: String,
    pub period_or_type: Option<String>,
}

pub struct BittradeWsAdapter {
    counters: Arc<WsCounters>,
    active: HashSet<WsSubscription>,
}

impl BittradeWsAdapter {
    pub fn new(counters: Arc<WsCounters>) -> Self {
        Self {
            counters,
            active: HashSet::new(),
        }
    }

    pub fn subscribe_command(&self, sub: &WsSubscription) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        Ok(format!(
            r#"{{\"sub\":\"{}\",\"id\":\"{}\"}}"#,
            render_channel(&spec.channel, sub),
            sub.symbol
        ))
    }

    pub fn unsubscribe_command(&self, sub: &WsSubscription) -> Result<String, UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        Ok(format!(
            r#"{{\"unsub\":\"{}\",\"id\":\"{}\"}}"#,
            render_channel(&spec.channel, sub),
            sub.symbol
        ))
    }

    pub async fn connect_and_subscribe<T: Transport>(
        &mut self,
        t: &T,
        sub: WsSubscription,
        key_id: Option<String>,
    ) -> Result<(), UcelError> {
        let spec = ws_specs()
            .into_iter()
            .find(|s| s.id == sub.channel_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown channel"))?;
        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: map_op(endpoint_id),
            venue: "bittrade".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let path = render_path(spec.path, &args.path_params)?;
        let req_path = with_query(format!("{}{}", spec.base_url, path), &args.query_params);
        let req = HttpRequest {
            method: spec.method.into(),
            path: req_path,
            body: args.body,
        };
        self.send_with_retry(req, ctx, endpoint_id).await
    }

    async fn send_with_retry(
        &self,
        req: HttpRequest,
        ctx: RequestContext,
        endpoint_id: &str,
    ) -> Result<BittradeRestResponse, UcelError> {
        let mut attempt = 0;
        loop {
            let send = tokio::time::timeout(
                self.timeout,
                self.transport.send_http(req.clone(), ctx.clone()),
            )
            .await;
            let resp = match send {
                Ok(v) => v,
                Err(_) => Err(UcelError::new(ErrorCode::Timeout, "request timeout")),
            };
            match resp {
                Ok(ok) => {
                    if ok.status == 429 {
                        let mut err = parse_error(&ok.body, ErrorCode::RateLimited);
                        if err.retry_after_ms.is_none() {
                            err.retry_after_ms = Some(self.retry_policy.base_delay_ms);
                        }
                        if attempt >= self.max_retries {
                            return Err(err);
                        }
                        let wait =
                            next_retry_delay_ms(&self.retry_policy, attempt, err.retry_after_ms);
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                        attempt += 1;
                        continue;
                    }
                    if ok.status >= 500 {
                        let err = UcelError::new(
                            ErrorCode::Upstream5xx,
                            format!("upstream status {}", ok.status),
                        );
                        if attempt >= self.max_retries {
                            return Err(err);
                        }
                        let wait = next_retry_delay_ms(&self.retry_policy, attempt, None);
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                        attempt += 1;
                        continue;
                    }
                    if ok.status >= 400 {
                        return Err(parse_error(&ok.body, ErrorCode::Internal));
                    }
                    return parse_success(endpoint_id, &ok.body);
                }
                Err(err) => {
                    let retryable = classify_error(&err.code) == RetryClass::Retryable;
                    if !retryable || attempt >= self.max_retries {
                        return Err(err);
                    }
                    let wait = next_retry_delay_ms(&self.retry_policy, attempt, err.retry_after_ms);
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                    attempt += 1;
                }
            }
        }
    }
}

fn render_path(
    path_template: &str,
    path_params: &BTreeMap<String, String>,
) -> Result<String, UcelError> {
    let mut rendered = path_template.to_string();
    for (k, v) in path_params {
        rendered = rendered.replace(&format!("{{{k}}}"), v);
    }
    if rendered.contains('{') {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "missing path param",
        ));
    }
    Ok(rendered)
}

fn with_query(path: String, query_params: &BTreeMap<String, String>) -> String {
    if query_params.is_empty() {
        return path;
    }
    let q = query_params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{path}?{q}")
}

fn parse_json<T: DeserializeOwned>(body: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(body)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse failed: {e}")))
}

fn parse_success(endpoint_id: &str, body: &Bytes) -> Result<BittradeRestResponse, UcelError> {
    if endpoint_id == "public.rest.common.currencys.get" {
        return Ok(BittradeRestResponse::ListString(parse_json(body)?));
    }
    if endpoint_id == "public.rest.common.timestamp.get" {
        return Ok(BittradeRestResponse::NumberData(parse_json(body)?));
    }
    if endpoint_id == "public.rest.market.kline.get"
        || endpoint_id == "public.rest.market.history.trade.get"
        || endpoint_id == "public.rest.market.tickers.get"
    {
        return Ok(BittradeRestResponse::TsListObject(parse_json(body)?));
    }
    if endpoint_id == "public.rest.market.depth.get"
        || endpoint_id == "public.rest.market.detail.merged.get"
        || endpoint_id == "public.rest.market.trade.get"
    {
        return Ok(BittradeRestResponse::TickObject(parse_json(body)?));
    }
    if endpoint_id == "private.rest.order.place.post"
        || endpoint_id == "private.rest.order.cancel.post"
        || endpoint_id == "private.rest.wallet.withdraw.create.post"
        || endpoint_id == "private.rest.wallet.withdraw.cancel.post"
        || endpoint_id == "private.rest.retail.order.place.post"
    {
        return Ok(BittradeRestResponse::DataObject(parse_json(body)?));
    }
    if endpoint_id == "other.rest.host.info" {
        return Ok(BittradeRestResponse::HostInfo(parse_json(body)?));
    }
    Ok(BittradeRestResponse::ListObject(parse_json(body)?))
}

#[derive(Debug, Deserialize)]
struct BittradeErrorBody {
    #[serde(rename = "err-code")]
    err_code: Option<String>,
    #[serde(rename = "err-msg")]
    err_msg: Option<String>,
    code: Option<String>,
    message: Option<String>,
    retry_after: Option<u64>,
    retry_after_ms: Option<u64>,
}

fn parse_error(body: &Bytes, fallback: ErrorCode) -> UcelError {
    if let Ok(e) = serde_json::from_slice::<BittradeErrorBody>(body) {
        let code_key = e.err_code.or(e.code).unwrap_or_default();
        let mapped = map_error_code(code_key.as_str(), fallback);
        let mut out = UcelError::new(
            mapped,
            e.err_msg
                .or(e.message)
                .unwrap_or_else(|| "bittrade error".into()),
        );
        out.retry_after_ms = e.retry_after_ms.or(e.retry_after);
        return out;
    }
    UcelError::new(fallback, "bittrade error")
}

fn map_error_code(code: &str, fallback: ErrorCode) -> ErrorCode {
    match code {
        "api-signature-not-valid" | "login-required" | "authentication-failed" => {
            ErrorCode::AuthFailed
        }
        "403" | "forbidden" | "permission-denied" => ErrorCode::PermissionDenied,
        "order-invalid" | "order-not-found" | "base-record-invalid" => ErrorCode::InvalidOrder,
        "429" | "too-many-requests" | "rate-limit" => ErrorCode::RateLimited,
        _ => fallback,
    }
}

fn map_op(id: &str) -> OpName {
    if id.contains("ticker") || id.contains("detail.merged") {
        OpName::FetchTicker
    } else if id.contains("trade") || id.contains("matchresults") {
        OpName::FetchTrades
    } else if id.contains("kline") {
        OpName::FetchKlines
    } else if id.contains("depth") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("balance") || id.contains("accounts") {
        OpName::FetchBalances
    } else if id.contains("place") {
        OpName::PlaceOrder
    } else if id.contains("cancel") {
        OpName::CancelOrder
    } else {
        OpName::FetchStatus
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
