use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{
    classify_error, enforce_auth_boundary, next_retry_delay_ms, HttpRequest, RequestContext,
    RetryClass, RetryPolicy, Transport,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

pub const REST_ENDPOINTS: [EndpointSpec; 27] = [
    EndpointSpec { id: "public.rest.common.symbols.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/common/symbols", requires_auth: false },
    EndpointSpec { id: "public.rest.common.currencys.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/common/currencys", requires_auth: false },
    EndpointSpec { id: "public.rest.common.timestamp.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/common/timestamp", requires_auth: false },
    EndpointSpec { id: "public.rest.market.kline.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/history/kline", requires_auth: false },
    EndpointSpec { id: "public.rest.market.depth.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/depth", requires_auth: false },
    EndpointSpec { id: "public.rest.market.detail.merged.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/detail/merged", requires_auth: false },
    EndpointSpec { id: "public.rest.market.trade.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/trade", requires_auth: false },
    EndpointSpec { id: "public.rest.market.history.trade.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/history/trade", requires_auth: false },
    EndpointSpec { id: "public.rest.market.tickers.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/market/tickers", requires_auth: false },
    EndpointSpec { id: "private.rest.account.accounts.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/account/accounts", requires_auth: true },
    EndpointSpec { id: "private.rest.account.balance.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/account/accounts/{account-id}/balance", requires_auth: true },
    EndpointSpec { id: "private.rest.order.place.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/place", requires_auth: true },
    EndpointSpec { id: "private.rest.order.cancel.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/{order-id}/submitcancel", requires_auth: true },
    EndpointSpec { id: "private.rest.order.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/{order-id}", requires_auth: true },
    EndpointSpec { id: "private.rest.order.list.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders", requires_auth: true },
    EndpointSpec { id: "private.rest.order.open.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/openOrders", requires_auth: true },
    EndpointSpec { id: "private.rest.order.matchresults.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/matchresults", requires_auth: true },
    EndpointSpec { id: "private.rest.order.matchresults.byorder.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/{order-id}/matchresults", requires_auth: true },
    EndpointSpec { id: "private.rest.order.batchcancel.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/batchcancel", requires_auth: true },
    EndpointSpec { id: "private.rest.order.batchcancel.open.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/order/orders/batchCancelOpenOrders", requires_auth: true },
    EndpointSpec { id: "private.rest.wallet.withdraw.create.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/dw/withdraw/api/create", requires_auth: true },
    EndpointSpec { id: "private.rest.wallet.withdraw.cancel.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/dw/withdraw-virtual/{withdraw-id}/cancel", requires_auth: true },
    EndpointSpec { id: "private.rest.wallet.depositwithdraw.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/query/deposit-withdraw", requires_auth: true },
    EndpointSpec { id: "private.rest.retail.maintain.time.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/retail/maintain/time", requires_auth: true },
    EndpointSpec { id: "private.rest.retail.order.place.post", method: "POST", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/retail/order/place", requires_auth: true },
    EndpointSpec { id: "private.rest.retail.order.list.get", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/v1/retail/order/list", requires_auth: true },
    EndpointSpec { id: "other.rest.host.info", method: "GET", base_url: "https://api-cloud.bittrade.co.jp", path: "/", requires_auth: false },
];

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BittradeScalar {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub type DynamicObject = BTreeMap<String, BittradeScalar>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListObjectResponse {
    pub status: String,
    pub data: Vec<DynamicObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListStringResponse {
    pub status: String,
    pub data: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NumberDataResponse {
    pub status: String,
    pub data: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TsListObjectResponse {
    pub status: String,
    pub ts: i64,
    pub data: Vec<DynamicObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickObjectResponse {
    pub status: String,
    pub ts: i64,
    pub tick: DynamicObject,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataObjectResponse {
    pub status: String,
    pub data: DynamicObject,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HostInfoResponse {
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum BittradeRestResponse {
    ListObject(ListObjectResponse),
    ListString(ListStringResponse),
    NumberData(NumberDataResponse),
    TsListObject(TsListObjectResponse),
    TickObject(TickObjectResponse),
    DataObject(DataObjectResponse),
    HostInfo(HostInfoResponse),
}

#[derive(Debug, Clone, Default)]
pub struct RequestArgs {
    pub path_params: BTreeMap<String, String>,
    pub query_params: BTreeMap<String, String>,
    pub body: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct BittradeRestClient<T: Transport> {
    transport: Arc<T>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl<T: Transport> BittradeRestClient<T> {
    pub fn new(transport: Arc<T>) -> Self {
        Self {
            transport,
            retry_policy: RetryPolicy {
                base_delay_ms: 50,
                max_delay_ms: 1_000,
                jitter_ms: 10,
                respect_retry_after: true,
            },
            timeout: Duration::from_secs(5),
            max_retries: 2,
        }
    }

    pub async fn execute(
        &self,
        endpoint_id: &str,
        args: RequestArgs,
        key_id: Option<String>,
    ) -> Result<BittradeRestResponse, UcelError> {
        let spec = REST_ENDPOINTS
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, "unknown endpoint"))?;
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
            let send = tokio::time::timeout(self.timeout, self.transport.send_http(req.clone(), ctx.clone())).await;
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
                        let wait = next_retry_delay_ms(&self.retry_policy, attempt, err.retry_after_ms);
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                        attempt += 1;
                        continue;
                    }
                    if ok.status >= 500 {
                        let err = UcelError::new(ErrorCode::Upstream5xx, format!("upstream status {}", ok.status));
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

fn render_path(path_template: &str, path_params: &BTreeMap<String, String>) -> Result<String, UcelError> {
    let mut rendered = path_template.to_string();
    for (k, v) in path_params {
        rendered = rendered.replace(&format!("{{{k}}}"), v);
    }
    if rendered.contains('{') {
        return Err(UcelError::new(ErrorCode::CatalogInvalid, "missing path param"));
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
        let mut out = UcelError::new(mapped, e.err_msg.or(e.message).unwrap_or_else(|| "bittrade error".into()));
        out.retry_after_ms = e.retry_after_ms.or(e.retry_after);
        return out;
    }
    UcelError::new(fallback, "bittrade error")
}

fn map_error_code(code: &str, fallback: ErrorCode) -> ErrorCode {
    match code {
        "api-signature-not-valid" | "login-required" | "authentication-failed" => ErrorCode::AuthFailed,
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
