use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use ucel_core::{ErrorCode, Exchange, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 10] = [
    EndpointSpec {
        id: "spot.public.rest.assets.list",
        method: "GET",
        path: "/0/public/Assets",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.public.rest.asset-pairs.list",
        method: "GET",
        path: "/0/public/AssetPairs",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.public.rest.ticker.get",
        method: "GET",
        path: "/0/public/Ticker",
        requires_auth: false,
    },
    EndpointSpec {
        id: "spot.private.rest.balance.get",
        method: "POST",
        path: "/0/private/Balance",
        requires_auth: true,
    },
    EndpointSpec {
        id: "spot.private.rest.order.add",
        method: "POST",
        path: "/0/private/AddOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "spot.private.rest.token.ws.get",
        method: "POST",
        path: "/0/private/GetWebSocketsToken",
        requires_auth: true,
    },
    EndpointSpec {
        id: "futures.public.rest.instruments.list",
        method: "GET",
        path: "/api/v3/instruments",
        requires_auth: false,
    },
    EndpointSpec {
        id: "futures.public.rest.tickers.list",
        method: "GET",
        path: "/api/v3/tickers",
        requires_auth: false,
    },
    EndpointSpec {
        id: "futures.private.rest.accounts.get",
        method: "GET",
        path: "/api/v3/accounts",
        requires_auth: true,
    },
    EndpointSpec {
        id: "futures.private.rest.order.send",
        method: "POST",
        path: "/api/v3/sendorder",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub enum KrakenRestResponse {
    SpotAssets(SpotAssetsResponse),
    SpotAssetPairs(SpotAssetPairsResponse),
    SpotTicker(SpotTickerResponse),
    SpotBalance(SpotBalanceResponse),
    SpotAddOrder(SpotAddOrderResponse),
    SpotWsToken(SpotWsTokenResponse),
    FuturesInstruments(FuturesInstrumentsResponse),
    FuturesTickers(FuturesTickersResponse),
    FuturesAccounts(FuturesAccountsResponse),
    FuturesSendOrder(FuturesSendOrderResponse),
}

#[derive(Clone)]
pub struct KrakenRestAdapter {
    spot_base_url: Arc<str>,
    futures_base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl KrakenRestAdapter {
    pub fn new(spot_base_url: impl Into<String>, futures_base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            spot_base_url: Arc::from(spot_base_url.into()),
            futures_base_url: Arc::from(futures_base_url.into()),
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

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<KrakenRestResponse, UcelError> {
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
            op: OpName::FetchStatus,
            venue: "kraken".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let base = if endpoint_id.starts_with("futures") {
            self.futures_base_url.as_ref()
        } else {
            self.spot_base_url.as_ref()
        };
        let req = HttpRequest {
            method: spec.method.into(),
            path: format!("{base}{}", spec.path),
            body,
        };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_kraken_http_error(response.status, &response.body));
        }

        let parsed = match endpoint_id {
            "spot.public.rest.assets.list" => {
                KrakenRestResponse::SpotAssets(parse_json(&response.body)?)
            }
            "spot.public.rest.asset-pairs.list" => {
                KrakenRestResponse::SpotAssetPairs(parse_json(&response.body)?)
            }
            "spot.public.rest.ticker.get" => {
                KrakenRestResponse::SpotTicker(parse_json(&response.body)?)
            }
            "spot.private.rest.balance.get" => {
                KrakenRestResponse::SpotBalance(parse_json(&response.body)?)
            }
            "spot.private.rest.order.add" => {
                KrakenRestResponse::SpotAddOrder(parse_json(&response.body)?)
            }
            "spot.private.rest.token.ws.get" => {
                KrakenRestResponse::SpotWsToken(parse_json(&response.body)?)
            }
            "futures.public.rest.instruments.list" => {
                KrakenRestResponse::FuturesInstruments(parse_json(&response.body)?)
            }
            "futures.public.rest.tickers.list" => {
                KrakenRestResponse::FuturesTickers(parse_json(&response.body)?)
            }
            "futures.private.rest.accounts.get" => {
                KrakenRestResponse::FuturesAccounts(parse_json(&response.body)?)
            }
            "futures.private.rest.order.send" => {
                KrakenRestResponse::FuturesSendOrder(parse_json(&response.body)?)
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

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct SpotKrakenErrorEnvelope {
    #[serde(default)]
    error: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FuturesKrakenErrorEnvelope {
    #[serde(default)]
    error: Option<String>,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
}

pub fn map_kraken_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|b| b.split("retry_after_ms=").nth(1))
            .and_then(|s| s.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let spot = serde_json::from_slice::<SpotKrakenErrorEnvelope>(body).ok();
    let futures = serde_json::from_slice::<FuturesKrakenErrorEnvelope>(body).ok();
    let code = spot
        .as_ref()
        .and_then(|v| v.error.first())
        .map(std::string::String::as_str)
        .or_else(|| futures.as_ref().and_then(|f| f.error.as_deref()))
        .or_else(|| futures.as_ref().and_then(|f| f.error_code.as_deref()))
        .unwrap_or_default();

    let mut err = match code {
        c if c.contains("EAPI:Invalid key") || c.contains("auth") => {
            UcelError::new(ErrorCode::AuthFailed, "authentication failed")
        }
        c if c.contains("EGeneral:Permission denied") => {
            UcelError::new(ErrorCode::PermissionDenied, "permission denied")
        }
        c if c.contains("EOrder:Invalid") || c.contains("invalidArgument") => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        c if c.contains("apiLimitExceeded") => {
            UcelError::new(ErrorCode::RateLimited, "rate limited")
        }
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("kraken http error status={status}"),
        ),
    };
    err.key_specific = matches!(
        err.code,
        ErrorCode::AuthFailed | ErrorCode::PermissionDenied
    );
    err
}

impl Exchange for KrakenRestAdapter {
    fn name(&self) -> &'static str {
        "kraken"
    }
    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("op {} not implemented", op),
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAssetsResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAssetPairsResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotTickerResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotBalanceResponse {
    pub error: Vec<String>,
    pub result: HashMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAddOrderResult {
    pub descr: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub txid: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotAddOrderResponse {
    pub error: Vec<String>,
    pub result: SpotAddOrderResult,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotWsTokenInner {
    pub token: String,
    pub expires: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotWsTokenResponse {
    pub error: Vec<String>,
    pub result: SpotWsTokenInner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesInstrumentsResponse {
    #[serde(rename = "serverTime")]
    pub server_time: String,
    pub instruments: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesTickersResponse {
    #[serde(rename = "serverTime")]
    pub server_time: String,
    pub tickers: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesAccountsResponse {
    pub accounts: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuturesSendOrderResponse {
    #[serde(rename = "sendStatus")]
    pub send_status: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{next_retry_delay_ms, HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        calls: AtomicUsize,
        responses: Mutex<HashMap<String, HttpResponse>>,
    }
    impl SpyTransport {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                responses: Mutex::new(HashMap::new()),
            }
        }
        async fn set_response(&self, path: &str, status: u16, body: &str) {
            self.responses.lock().await.insert(
                path.into(),
                HttpResponse {
                    status,
                    body: Bytes::copy_from_slice(body.as_bytes()),
                },
            );
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }
    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.responses.lock().await.remove(&req.path).unwrap())
        }
        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream { connected: true })
        }
    }

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_all_endpoints_are_covered_by_fixture_driven_tests() {
        let transport = SpyTransport::new();
        let adapter = KrakenRestAdapter::new(
            "https://api.kraken.test",
            "https://futures.kraken.test/derivatives",
        );
        for spec in KrakenRestAdapter::endpoint_specs() {
            let filename = format!("{}.json", spec.id);
            let path = if spec.id.starts_with("futures") {
                format!("https://futures.kraken.test/derivatives{}", spec.path)
            } else {
                format!("https://api.kraken.test{}", spec.path)
            };
            transport
                .set_response(&path, 200, &fixture(&filename))
                .await;
            let key = if spec.requires_auth {
                Some("k-1".to_string())
            } else {
                None
            };
            assert!(
                adapter
                    .execute_rest(&transport, spec.id, None, key)
                    .await
                    .is_ok(),
                "failed id={}",
                spec.id
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_rejects_without_auth_and_transport_is_not_called() {
        let transport = SpyTransport::new();
        let adapter = KrakenRestAdapter::new(
            "https://api.kraken.test",
            "https://futures.kraken.test/derivatives",
        );
        let err = adapter
            .execute_rest(&transport, "spot.private.rest.balance.get", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[test]
    fn maps_kraken_errors_to_ucel_error_codes() {
        let auth = map_kraken_http_error(401, br#"{"error":["EAPI:Invalid key"]}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let invalid = map_kraken_http_error(400, br#"{"error":["EOrder:Invalid price"]}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);

        let rate = map_kraken_http_error(429, b"retry_after_ms=1500");
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(1500));
    }

    #[test]
    fn retry_policy_respects_retry_after_for_429() {
        let policy = RetryPolicy {
            base_delay_ms: 100,
            max_delay_ms: 4_000,
            jitter_ms: 50,
            respect_retry_after: true,
        };
        assert_eq!(next_retry_delay_ms(&policy, 3, Some(777)), 777);
    }

    #[test]
    fn kraken_coverage_manifest_has_no_rest_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/kraken.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        let rest_entries: Vec<_> = manifest
            .entries
            .iter()
            .filter(|e| e.id.contains(".rest."))
            .collect();
        assert!(!rest_entries.is_empty());
        for e in rest_entries {
            assert!(e.implemented, "rest id not implemented: {}", e.id);
            assert!(e.tested, "rest id not tested: {}", e.id);
        }
        let _ = evaluate_coverage_gate(&manifest);
    }
}
