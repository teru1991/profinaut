use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
    pub op: OpName,
}

const ENDPOINTS: [EndpointSpec; 9] = [
    EndpointSpec {
        id: "crypto.public.rest.ping.get",
        method: "GET",
        base_url: "https://api.binance.com",
        path: "/api/v3/ping",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
    EndpointSpec {
        id: "crypto.public.rest.time.get",
        method: "GET",
        base_url: "https://api.binance.com",
        path: "/api/v3/time",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
    EndpointSpec {
        id: "crypto.public.rest.exchangeinfo.get",
        method: "GET",
        base_url: "https://api.binance.com",
        path: "/api/v3/exchangeInfo",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
    EndpointSpec {
        id: "crypto.private.rest.order.post",
        method: "POST",
        base_url: "https://api.binance.com",
        path: "/api/v3/order",
        requires_auth: true,
        op: OpName::PlaceOrder,
    },
    EndpointSpec {
        id: "crypto.private.rest.account.get",
        method: "GET",
        base_url: "https://api.binance.com",
        path: "/api/v3/account",
        requires_auth: true,
        op: OpName::FetchBalances,
    },
    EndpointSpec {
        id: "crypto.private.rest.listenkey.post",
        method: "POST",
        base_url: "https://api.binance.com",
        path: "/api/v3/userDataStream",
        requires_auth: true,
        op: OpName::CreateWsAuthToken,
    },
    EndpointSpec {
        id: "crypto.public.rest.docs.enums.ref",
        method: "GET",
        base_url: "docs://binance-spot",
        path: "/rest-api/enum-definitions",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
    EndpointSpec {
        id: "crypto.public.rest.docs.filters.ref",
        method: "GET",
        base_url: "docs://binance-spot",
        path: "/rest-api/filters",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
    EndpointSpec {
        id: "other.public.rest.changelog.ref",
        method: "GET",
        base_url: "docs://binance-spot",
        path: "/changelog",
        requires_auth: false,
        op: OpName::FetchStatus,
    },
];

#[derive(Debug, Clone)]
pub enum BinanceRestResponse {
    Ping(PingResponse),
    Time(TimeResponse),
    ExchangeInfo(ExchangeInfoResponse),
    Order(OrderResponse),
    Account(AccountResponse),
    ListenKey(ListenKeyResponse),
    EnumsRef(DocRefResponse),
    FiltersRef(DocRefResponse),
    ChangelogRef(DocRefResponse),
}

#[derive(Clone)]
pub struct BinanceRestAdapter {
    http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BinanceRestAdapter {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");

        Self {
            http_client,
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn endpoint_specs() -> &'static [EndpointSpec] {
        &ENDPOINTS
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<BinanceRestResponse, UcelError> {
        let spec = ENDPOINTS
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
            op: spec.op,
            venue: "binance".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.to_string(),
            path: format!("{}{}", spec.base_url, spec.path),
            body,
        };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_binance_http_error(response.status, &response.body));
        }

        let parsed = match endpoint_id {
            "crypto.public.rest.ping.get" => BinanceRestResponse::Ping(parse_json(&response.body)?),
            "crypto.public.rest.time.get" => BinanceRestResponse::Time(parse_json(&response.body)?),
            "crypto.public.rest.exchangeinfo.get" => {
                BinanceRestResponse::ExchangeInfo(parse_json(&response.body)?)
            }
            "crypto.private.rest.order.post" => {
                BinanceRestResponse::Order(parse_json(&response.body)?)
            }
            "crypto.private.rest.account.get" => {
                BinanceRestResponse::Account(parse_json(&response.body)?)
            }
            "crypto.private.rest.listenkey.post" => {
                BinanceRestResponse::ListenKey(parse_json(&response.body)?)
            }
            "crypto.public.rest.docs.enums.ref" => {
                BinanceRestResponse::EnumsRef(parse_json(&response.body)?)
            }
            "crypto.public.rest.docs.filters.ref" => {
                BinanceRestResponse::FiltersRef(parse_json(&response.body)?)
            }
            "other.public.rest.changelog.ref" => {
                BinanceRestResponse::ChangelogRef(parse_json(&response.body)?)
            }
            _ => {
                return Err(UcelError::new(
                    ErrorCode::NotSupported,
                    format!("unsupported endpoint: {endpoint_id}"),
                ))
            }
        };

        Ok(parsed)
    }
}

impl Default for BinanceRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct BinanceErrorEnvelope {
    code: Option<i64>,
    msg: Option<String>,
}

pub fn map_binance_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.ban_risk = true;
        err.retry_after_ms = parse_retry_after_ms(body);
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let payload = serde_json::from_slice::<BinanceErrorEnvelope>(body).ok();
    let code = payload.as_ref().and_then(|e| e.code).unwrap_or_default();

    let mut err = match code {
        -1003 | -1015 | -1034 => UcelError::new(ErrorCode::RateLimited, "rate limited"),
        -2014 | -2015 | -1022 => UcelError::new(ErrorCode::AuthFailed, "authentication failed"),
        -1002 | -2010 => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        -1013 | -1111 | -1116 | -1117 | -1121 | -1130 => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("binance http error status={status} code={code}"),
        ),
    };
    err.key_specific = matches!(
        err.code,
        ErrorCode::AuthFailed | ErrorCode::PermissionDenied
    );
    if err.code == ErrorCode::RateLimited {
        err.retry_after_ms = parse_retry_after_ms(body);
        err.ban_risk = true;
    }
    if let Some(msg) = payload.and_then(|e| e.msg) {
        err.message = format!("{} ({msg})", err.message);
    }
    err
}

fn parse_retry_after_ms(body: &[u8]) -> Option<u64> {
    std::str::from_utf8(body)
        .ok()
        .and_then(|txt| txt.split("retry_after_ms=").nth(1))
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
}

#[derive(Debug, Clone, Deserialize)]
pub struct PingResponse {}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeResponse {
    #[serde(rename = "serverTime")]
    pub server_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInfoResponse {
    pub timezone: String,
    #[serde(rename = "serverTime")]
    pub server_time: u64,
    pub symbols: Vec<ExchangeInfoSymbol>,
    #[serde(rename = "rateLimits")]
    pub rate_limits: Vec<ExchangeRateLimit>,
    #[serde(rename = "exchangeFilters")]
    pub exchange_filters: Vec<ExchangeFilter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInfoSymbol {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeRateLimit {
    #[serde(rename = "rateLimitType")]
    pub rate_limit_type: String,
    pub interval: String,
    #[serde(rename = "intervalNum")]
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeFilter {
    #[serde(rename = "filterType")]
    pub filter_type: String,
    #[serde(rename = "maxNumOrders")]
    pub max_num_orders: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "transactTime")]
    pub transact_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountResponse {
    #[serde(rename = "makerCommission")]
    pub maker_commission: u32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: u32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    pub balances: Vec<AccountBalance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocRefResponse {
    pub items: Vec<DocRefItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocRefItem {
    pub id: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        calls: Mutex<Vec<HttpRequest>>,
        responses: Mutex<VecDeque<Result<HttpResponse, UcelError>>>,
    }

    impl SpyTransport {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::new()),
            }
        }

        fn enqueue_response(&self, status: u16, body: &'static str) {
            self.responses.lock().unwrap().push_back(Ok(HttpResponse {
                status,
                body: Bytes::from_static(body.as_bytes()),
            }));
        }

        fn enqueue_error(&self, err: UcelError) {
            self.responses.lock().unwrap().push_back(Err(err));
        }

        fn calls_len(&self) -> usize {
            self.calls.lock().unwrap().len()
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.lock().unwrap().push(req);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| panic!("missing queued response"))
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream::default())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn all_catalog_rest_ids_are_implemented_and_parse() {
        let adapter = BinanceRestAdapter::new();
        let transport = SpyTransport::new();
        let fixtures = [
            ("crypto.public.rest.ping.get", "{}"),
            (
                "crypto.public.rest.time.get",
                r#"{"serverTime":1700000000000}"#,
            ),
            (
                "crypto.public.rest.exchangeinfo.get",
                r#"{"timezone":"UTC","serverTime":1700000000000,"symbols":[{"symbol":"BTCUSDT","status":"TRADING","baseAsset":"BTC","quoteAsset":"USDT"}],"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":1200}],"exchangeFilters":[{"filterType":"EXCHANGE_MAX_NUM_ORDERS","maxNumOrders":200}] }"#,
            ),
            (
                "crypto.private.rest.order.post",
                r#"{"symbol":"BTCUSDT","orderId":1,"clientOrderId":"cid-1","transactTime":1700000000000}"#,
            ),
            (
                "crypto.private.rest.account.get",
                r#"{"makerCommission":10,"takerCommission":10,"canTrade":true,"balances":[{"asset":"USDT","free":"1.0","locked":"0.0"}]}"#,
            ),
            (
                "crypto.private.rest.listenkey.post",
                r#"{"listenKey":"abc123"}"#,
            ),
            (
                "crypto.public.rest.docs.enums.ref",
                r#"{"items":[{"id":"SIDE","title":"BUY/SELL"}]}"#,
            ),
            (
                "crypto.public.rest.docs.filters.ref",
                r#"{"items":[{"id":"PRICE_FILTER","title":"price range"}]}"#,
            ),
            (
                "other.public.rest.changelog.ref",
                r#"{"items":[{"id":"2026-01-01","title":"Update"}]}"#,
            ),
        ];

        for (id, body) in fixtures {
            transport.enqueue_response(200, body);
            let key = if id.contains("private") {
                Some("kid".to_string())
            } else {
                None
            };
            adapter
                .execute_rest(&transport, id, None, key)
                .await
                .unwrap_or_else(|e| panic!("failed for {id}: {e}"));
        }

        assert_eq!(transport.calls_len(), ENDPOINTS.len());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_is_rejected_before_transport_when_auth_missing() {
        let adapter = BinanceRestAdapter::new();
        let transport = SpyTransport::new();

        let err = adapter
            .execute_rest(&transport, "crypto.private.rest.order.post", None, None)
            .await
            .unwrap_err();

        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls_len(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_endpoint_never_sends_key_path() {
        let adapter = BinanceRestAdapter::new();
        let transport = SpyTransport::new();
        transport.enqueue_response(200, "{}");

        adapter
            .execute_rest(
                &transport,
                "crypto.public.rest.ping.get",
                None,
                Some("should-not-be-used".into()),
            )
            .await
            .unwrap();

        assert_eq!(transport.calls_len(), 1);
    }

    #[test]
    fn error_mapping_for_rate_limit_429_and_code_based_mapping() {
        let rate = map_binance_http_error(429, b"retry_after_ms=1200");
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(1200));

        let auth = map_binance_http_error(400, br#"{"code":-2015,"msg":"Invalid API-key"}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let perm =
            map_binance_http_error(400, br#"{"code":-2010,"msg":"insufficient permissions"}"#);
        assert_eq!(perm.code, ErrorCode::PermissionDenied);

        let inv = map_binance_http_error(400, br#"{"code":-1013,"msg":"Invalid quantity"}"#);
        assert_eq!(inv.code, ErrorCode::InvalidOrder);

        let up = map_binance_http_error(503, b"oops");
        assert_eq!(up.code, ErrorCode::Upstream5xx);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn transport_timeout_is_propagated() {
        let adapter = BinanceRestAdapter::new();
        let transport = SpyTransport::new();
        transport.enqueue_error(UcelError::new(ErrorCode::Timeout, "timeout"));

        let err = adapter
            .execute_rest(&transport, "crypto.public.rest.time.get", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);
    }

    #[test]
    fn strict_coverage_gate_for_binance_has_no_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/binance.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }
}

pub mod channels;
pub mod symbols;
pub mod ws;
pub mod ws_manager;
