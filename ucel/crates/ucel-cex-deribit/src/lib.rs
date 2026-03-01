use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ucel_core::decimal::serde::deserialize_decimal_observation;
use ucel_core::{Decimal, ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct DeribitRestAdapter {
    pub base_url: Arc<str>,
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl DeribitRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Arc::from(base_url.into()),
            endpoints: Arc::new(load_endpoint_specs()),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn endpoint_specs(&self) -> &[EndpointSpec] {
        &self.endpoints
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        request: DeribitRestRequest,
        key_id: Option<String>,
    ) -> Result<DeribitRestResponse, UcelError> {
        let endpoint_id = request.endpoint_id();
        let spec = self
            .endpoints
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| {
                UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unknown endpoint: {endpoint_id}"),
                )
            })?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: request.op_name(),
            venue: "deribit".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let rpc = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: &spec.method,
            params: request.params(),
        };
        let body = serde_json::to_vec(&rpc)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json encode error: {e}")))?;

        let response = transport
            .send_http(
                HttpRequest {
                    method: "POST".into(),
                    path: self.base_url.to_string(),
                    body: Some(Bytes::from(body)),
                },
                ctx,
            )
            .await?;

        if response.status >= 400 {
            return Err(map_deribit_http_error(response.status, &response.body));
        }

        parse_jsonrpc_response(&request, &response.body)
    }
}

#[derive(Debug, Clone)]
pub enum DeribitRestRequest {
    PublicGetInstruments(PublicGetInstrumentsParams),
    PublicTicker(PublicTickerParams),
    PublicGetOrderBook(PublicGetOrderBookParams),
    PublicGetTradingViewChartData(PublicGetTradingViewChartDataParams),
    PublicAuth(PublicAuthParams),
    PrivateGetAccountSummary(PrivateGetAccountSummaryParams),
    PrivateBuy(PrivateOrderParams),
    PrivateSell(PrivateOrderParams),
    PrivateCancel(PrivateCancelParams),
}

impl DeribitRestRequest {
    pub fn endpoint_id(&self) -> &'static str {
        match self {
            Self::PublicGetInstruments(_) => {
                "jsonrpc.http.public.market_data.public_get_instruments"
            }
            Self::PublicTicker(_) => "jsonrpc.http.public.market_data.public_ticker",
            Self::PublicGetOrderBook(_) => "jsonrpc.http.public.market_data.public_get_order_book",
            Self::PublicGetTradingViewChartData(_) => {
                "jsonrpc.http.public.market_data.public_get_tradingview_chart_data"
            }
            Self::PublicAuth(_) => "jsonrpc.http.private.auth.public_auth",
            Self::PrivateGetAccountSummary(_) => {
                "jsonrpc.http.private.account.private_get_account_summary"
            }
            Self::PrivateBuy(_) => "jsonrpc.http.private.trading.private_buy",
            Self::PrivateSell(_) => "jsonrpc.http.private.trading.private_sell",
            Self::PrivateCancel(_) => "jsonrpc.http.private.trading.private_cancel",
        }
    }

    fn params(&self) -> DeribitParams {
        match self {
            Self::PublicGetInstruments(v) => DeribitParams::PublicGetInstruments(v.clone()),
            Self::PublicTicker(v) => DeribitParams::PublicTicker(v.clone()),
            Self::PublicGetOrderBook(v) => DeribitParams::PublicGetOrderBook(v.clone()),
            Self::PublicGetTradingViewChartData(v) => {
                DeribitParams::PublicGetTradingViewChartData(v.clone())
            }
            Self::PublicAuth(v) => DeribitParams::PublicAuth(v.clone()),
            Self::PrivateGetAccountSummary(v) => DeribitParams::PrivateGetAccountSummary(v.clone()),
            Self::PrivateBuy(v) => DeribitParams::PrivateOrder(v.clone()),
            Self::PrivateSell(v) => DeribitParams::PrivateOrder(v.clone()),
            Self::PrivateCancel(v) => DeribitParams::PrivateCancel(v.clone()),
        }
    }

    fn op_name(&self) -> OpName {
        match self {
            Self::PublicGetInstruments(_) => OpName::FetchStatus,
            Self::PublicTicker(_) => OpName::FetchTicker,
            Self::PublicGetOrderBook(_) => OpName::FetchOrderbookSnapshot,
            Self::PublicGetTradingViewChartData(_) => OpName::FetchKlines,
            Self::PublicAuth(_) => OpName::CreateWsAuthToken,
            Self::PrivateGetAccountSummary(_) => OpName::FetchMarginStatus,
            Self::PrivateBuy(_) | Self::PrivateSell(_) => OpName::PlaceOrder,
            Self::PrivateCancel(_) => OpName::CancelOrder,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum DeribitParams {
    PublicGetInstruments(PublicGetInstrumentsParams),
    PublicTicker(PublicTickerParams),
    PublicGetOrderBook(PublicGetOrderBookParams),
    PublicGetTradingViewChartData(PublicGetTradingViewChartDataParams),
    PublicAuth(PublicAuthParams),
    PrivateGetAccountSummary(PrivateGetAccountSummaryParams),
    PrivateOrder(PrivateOrderParams),
    PrivateCancel(PrivateCancelParams),
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicGetInstrumentsParams {
    pub currency: String,
    pub kind: Option<String>,
    pub expired: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicTickerParams {
    pub instrument_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicGetOrderBookParams {
    pub instrument_name: String,
    pub depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicGetTradingViewChartDataParams {
    pub instrument_name: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicAuthParams {
    pub grant_type: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivateGetAccountSummaryParams {
    pub currency: String,
    pub extended: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivateOrderParams {
    pub instrument_name: String,
    pub amount: Decimal,
    #[serde(rename = "type")]
    pub order_type: String,
    pub price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivateCancelParams {
    pub order_id: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a, T: Serialize> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: T,
}

#[derive(Debug, Deserialize)]
struct JsonRpcEnvelope<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<T>,
    error: Option<DeribitErrorPayload>,
}

#[derive(Debug, Deserialize)]
struct DeribitErrorPayload {
    code: i64,
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitInstrument {
    pub instrument_name: String,
    pub kind: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub settlement_currency: String,
    #[serde(deserialize_with = "deserialize_decimal_observation")]
    pub tick_size: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitTicker {
    pub instrument_name: String,
    pub timestamp: i64,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub last_price: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub best_bid_price: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub best_ask_price: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub mark_price: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitOrderBook {
    pub timestamp: i64,
    pub instrument_name: String,
    #[serde(deserialize_with = "deserialize_levels_observation")]
    pub bids: Vec<[Decimal; 2]>,
    #[serde(deserialize_with = "deserialize_levels_observation")]
    pub asks: Vec<[Decimal; 2]>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitChartData {
    pub ticks: Vec<i64>,
    #[serde(deserialize_with = "deserialize_vec_decimal_observation")]
    pub open: Vec<Decimal>,
    #[serde(deserialize_with = "deserialize_vec_decimal_observation")]
    pub high: Vec<Decimal>,
    #[serde(deserialize_with = "deserialize_vec_decimal_observation")]
    pub low: Vec<Decimal>,
    #[serde(deserialize_with = "deserialize_vec_decimal_observation")]
    pub close: Vec<Decimal>,
    #[serde(deserialize_with = "deserialize_vec_decimal_observation")]
    pub volume: Vec<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitAuthToken {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitAccountSummary {
    pub currency: String,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub balance: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub available_funds: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitOrderResult {
    pub order_id: String,
    pub instrument_name: String,
    pub direction: Option<String>,
    pub order_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub amount: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_opt_decimal_observation")]
    pub price: Option<Decimal>,
    pub order_state: Option<String>,
}

fn deserialize_opt_decimal_observation<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<GDecimal>::deserialize(deserializer).map(|x| x.map(|v| v.0))
}

fn deserialize_vec_decimal_observation<'de, D>(deserializer: D) -> Result<Vec<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<GDecimal>::deserialize(deserializer).map(|vals| vals.into_iter().map(|v| v.0).collect())
}

fn deserialize_levels_observation<'de, D>(deserializer: D) -> Result<Vec<[Decimal; 2]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let vals = Vec::<[GDecimal; 2]>::deserialize(deserializer)?;
    Ok(vals.into_iter().map(|x| [x[0].0, x[1].0]).collect())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeribitCancelResult {
    pub order_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeribitRestResponse {
    PublicGetInstruments(Vec<DeribitInstrument>),
    PublicTicker(DeribitTicker),
    PublicGetOrderBook(DeribitOrderBook),
    PublicGetTradingViewChartData(DeribitChartData),
    PublicAuth(DeribitAuthToken),
    PrivateGetAccountSummary(DeribitAccountSummary),
    PrivateBuy(DeribitOrderResult),
    PrivateSell(DeribitOrderResult),
    PrivateCancel(DeribitCancelResult),
}

fn parse_jsonrpc_response(
    request: &DeribitRestRequest,
    body: &[u8],
) -> Result<DeribitRestResponse, UcelError> {
    match request {
        DeribitRestRequest::PublicGetInstruments(_) => {
            decode_result(body).map(DeribitRestResponse::PublicGetInstruments)
        }
        DeribitRestRequest::PublicTicker(_) => {
            decode_result(body).map(DeribitRestResponse::PublicTicker)
        }
        DeribitRestRequest::PublicGetOrderBook(_) => {
            decode_result(body).map(DeribitRestResponse::PublicGetOrderBook)
        }
        DeribitRestRequest::PublicGetTradingViewChartData(_) => {
            decode_result(body).map(DeribitRestResponse::PublicGetTradingViewChartData)
        }
        DeribitRestRequest::PublicAuth(_) => {
            decode_result(body).map(DeribitRestResponse::PublicAuth)
        }
        DeribitRestRequest::PrivateGetAccountSummary(_) => {
            decode_result(body).map(DeribitRestResponse::PrivateGetAccountSummary)
        }
        DeribitRestRequest::PrivateBuy(_) => {
            decode_result(body).map(DeribitRestResponse::PrivateBuy)
        }
        DeribitRestRequest::PrivateSell(_) => {
            decode_result(body).map(DeribitRestResponse::PrivateSell)
        }
        DeribitRestRequest::PrivateCancel(_) => {
            decode_result(body).map(DeribitRestResponse::PrivateCancel)
        }
    }
}

fn decode_result<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, UcelError> {
    let envelope: JsonRpcEnvelope<T> = serde_json::from_slice(body)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;
    if let Some(error) = envelope.error {
        return Err(map_deribit_rpc_error(error));
    }
    envelope
        .result
        .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing result in JSON-RPC response"))
}

pub fn map_deribit_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|text| text.split("retry_after_ms=").nth(1))
            .and_then(|num| num.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    if let Ok(envelope) = serde_json::from_slice::<JsonRpcEnvelope<DeribitCancelResult>>(body) {
        if let Some(error) = envelope.error {
            return map_deribit_rpc_error(error);
        }
    }

    UcelError::new(
        ErrorCode::Network,
        format!("deribit http error status={status}"),
    )
}

fn map_deribit_rpc_error(error: DeribitErrorPayload) -> UcelError {
    let code = match error.code {
        10028 => ErrorCode::RateLimited,
        9999 | 10000 | 10001 | 10002 | 13009 => ErrorCode::AuthFailed,
        13021 => ErrorCode::PermissionDenied,
        10004 | 10005 | 10007 | 11035 | 11036 | 12003 | 12004 => ErrorCode::InvalidOrder,
        _ => ErrorCode::Network,
    };
    UcelError::new(code, format!("deribit rpc error code={}", error.code))
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rpc_http_methods: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    auth: CatalogAuth,
}

#[derive(Debug, Deserialize)]
struct CatalogAuth {
    #[serde(rename = "type")]
    auth_type: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/deribit/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid deribit catalog");
    catalog
        .rpc_http_methods
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            requires_auth: entry.auth.auth_type.eq_ignore_ascii_case("token"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        calls: AtomicUsize,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
        contexts: Mutex<Vec<RequestContext>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                response: Mutex::new(Some(resp)),
                contexts: Mutex::new(Vec::new()),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.contexts.lock().unwrap().push(ctx);
            self.response.lock().unwrap().take().unwrap()
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream::default())
        }
    }

    fn adapter() -> DeribitRestAdapter {
        DeribitRestAdapter::new("https://www.deribit.com/api/v2")
    }

    #[test]
    fn endpoint_specs_cover_all_catalog_rows() {
        let all = adapter();
        let ids: BTreeSet<_> = all.endpoint_specs().iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids.len(), 9);
        assert!(ids.contains("jsonrpc.http.private.trading.private_cancel"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_request_without_auth_is_rejected_preflight() {
        let adapter = adapter();
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(b"{}"),
        }));
        let err = adapter
            .execute_rest(
                &transport,
                DeribitRestRequest::PrivateCancel(PrivateCancelParams {
                    order_id: "abc".into(),
                }),
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_request_has_no_key_path() {
        let adapter = adapter();
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(
                br#"{"jsonrpc":"2.0","id":1,"result":{"instrument_name":"BTC-PERPETUAL","timestamp":1}}"#,
            ),
        }));
        let _ = adapter
            .execute_rest(
                &transport,
                DeribitRestRequest::PublicTicker(PublicTickerParams {
                    instrument_name: "BTC-PERPETUAL".into(),
                }),
                None,
            )
            .await
            .unwrap();
        let contexts = transport.contexts.lock().unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].key_id, None);
    }

    #[test]
    fn maps_rpc_codes_without_message_matching() {
        let err = map_deribit_rpc_error(DeribitErrorPayload {
            code: 11035,
            message: "ignored".into(),
        });
        assert_eq!(err.code, ErrorCode::InvalidOrder);
    }

    #[test]
    fn maps_429_with_retry_after() {
        let err = map_deribit_http_error(429, b"retry_after_ms=250");
        assert_eq!(err.code, ErrorCode::RateLimited);
        assert_eq!(err.retry_after_ms, Some(250));
    }
}

pub mod channels;
pub mod symbols;
pub mod ws_manager;
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GDecimal(#[serde(deserialize_with = "deserialize_decimal_observation")] Decimal);
