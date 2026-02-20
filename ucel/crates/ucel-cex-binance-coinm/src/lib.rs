use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ucel_core::{ErrorCode, Exchange, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub base_url: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 7] = [
    EndpointSpec {
        id: "coinm.public.rest.general.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/general-info",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.errors.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/error-code",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.common.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/common-definition",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.public.rest.market.ref",
        method: "GET",
        base_url: "docs://binance-coinm",
        path: "/market-data/rest-api",
        requires_auth: false,
    },
    EndpointSpec {
        id: "coinm.private.rest.trade.ref",
        method: "POST/GET/DELETE",
        base_url: "docs://binance-coinm",
        path: "/trade/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "coinm.private.rest.account.ref",
        method: "GET/POST",
        base_url: "docs://binance-coinm",
        path: "/account/rest-api",
        requires_auth: true,
    },
    EndpointSpec {
        id: "coinm.private.rest.listenkey.ref",
        method: "POST/PUT/DELETE",
        base_url: "https://dapi.binance.com",
        path: "/dapi/v1/listenKey",
        requires_auth: true,
    },
];

#[derive(Debug, Clone)]
pub enum BinanceCoinmRestResponse {
    General(RefPageResponse),
    Errors(RefPageResponse),
    Common(RefPageResponse),
    Market(RefPageResponse),
    Trade(RefPageResponse),
    Account(RefPageResponse),
    ListenKey(ListenKeyResponse),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RefPageResponse {
    pub section: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

#[derive(Clone)]
pub struct BinanceCoinmRestAdapter {
    docs_base_url: Arc<str>,
    api_base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BinanceCoinmRestAdapter {
    pub fn new(docs_base_url: impl Into<String>, api_base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            docs_base_url: Arc::from(docs_base_url.into()),
            api_base_url: Arc::from(api_base_url.into()),
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
    ) -> Result<BinanceCoinmRestResponse, UcelError> {
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
            venue: "binance-coinm".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let base = if spec.base_url == "https://dapi.binance.com" {
            self.api_base_url.as_ref()
        } else {
            self.docs_base_url.as_ref()
        };

        let req = HttpRequest {
            method: spec.method.into(),
            path: format!("{base}{}", spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_binance_coinm_http_error(
                response.status,
                &response.body,
            ));
        }

        let parsed = match endpoint_id {
            "coinm.public.rest.general.ref" => {
                BinanceCoinmRestResponse::General(parse_json(&response.body)?)
            }
            "coinm.public.rest.errors.ref" => {
                BinanceCoinmRestResponse::Errors(parse_json(&response.body)?)
            }
            "coinm.public.rest.common.ref" => {
                BinanceCoinmRestResponse::Common(parse_json(&response.body)?)
            }
            "coinm.public.rest.market.ref" => {
                BinanceCoinmRestResponse::Market(parse_json(&response.body)?)
            }
            "coinm.private.rest.trade.ref" => {
                BinanceCoinmRestResponse::Trade(parse_json(&response.body)?)
            }
            "coinm.private.rest.account.ref" => {
                BinanceCoinmRestResponse::Account(parse_json(&response.body)?)
            }
            "coinm.private.rest.listenkey.ref" => {
                BinanceCoinmRestResponse::ListenKey(parse_json(&response.body)?)
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

#[derive(Debug, Deserialize)]
struct BinanceCoinmErrorEnvelope {
    code: Option<i64>,
}

pub fn map_binance_coinm_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|b| b.split("retry_after_ms=").nth(1))
            .and_then(|v| v.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let code = serde_json::from_slice::<BinanceCoinmErrorEnvelope>(body)
        .ok()
        .and_then(|v| v.code)
        .unwrap_or_default();

    let mut err = match code {
        -2015 | -2014 | -1022 => UcelError::new(ErrorCode::AuthFailed, "authentication failed"),
        -2010 | -2011 | -1116 | -1111 => UcelError::new(ErrorCode::InvalidOrder, "invalid order"),
        -1003 | -1015 => UcelError::new(ErrorCode::RateLimited, "rate limited"),
        -1002 | -2017 => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("binance-coinm http error status={status}"),
        ),
    };
    err.key_specific = matches!(
        err.code,
        ErrorCode::AuthFailed | ErrorCode::PermissionDenied
    );
    err
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

impl Exchange for BinanceCoinmRestAdapter {
    fn name(&self) -> &'static str {
        "binance-coinm"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("op {} not implemented", op),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Debug, Deserialize)]
    struct CoverageManifest {
        venue: String,
        strict: bool,
        entries: Vec<CoverageEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct CoverageEntry {
        id: String,
        implemented: bool,
        tested: bool,
    }

    struct SpyTransport {
        calls: AtomicUsize,
        key_ids: Mutex<Vec<Option<String>>>,
        responses: Mutex<HashMap<String, HttpResponse>>,
    }

    impl SpyTransport {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                key_ids: Mutex::new(Vec::new()),
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
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.key_ids.lock().await.push(ctx.key_id.clone());
            self.responses
                .lock()
                .await
                .remove(&req.path)
                .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing mocked response"))
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
    async fn rest_contract_all_catalog_endpoints_parse_with_fixtures() {
        let transport = SpyTransport::new();
        let adapter =
            BinanceCoinmRestAdapter::new("https://docs.test/binance-coinm", "https://dapi.test");

        for spec in BinanceCoinmRestAdapter::endpoint_specs() {
            let filename = format!("{}.json", spec.id);
            let base = if spec.base_url == "https://dapi.binance.com" {
                "https://dapi.test"
            } else {
                "https://docs.test/binance-coinm"
            };
            let path = format!("{base}{}", spec.path);
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

        let keys = transport.key_ids.lock().await.clone();
        assert!(
            keys.iter().any(|k| k.is_none()),
            "public route must use no key path"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_auth_and_transport_is_not_called() {
        let transport = SpyTransport::new();
        let adapter =
            BinanceCoinmRestAdapter::new("https://docs.test/binance-coinm", "https://dapi.test");
        let err = adapter
            .execute_rest(&transport, "coinm.private.rest.trade.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[test]
    fn maps_binance_coinm_errors_by_code() {
        let auth = map_binance_coinm_http_error(401, br#"{"code":-2015,"msg":"bad key"}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let perm = map_binance_coinm_http_error(403, br#"{"code":-1002,"msg":"permission"}"#);
        assert_eq!(perm.code, ErrorCode::PermissionDenied);

        let invalid = map_binance_coinm_http_error(400, br#"{"code":-1111,"msg":"bad order"}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);

        let rate = map_binance_coinm_http_error(429, b"retry_after_ms=321");
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(321));

        let upstream = map_binance_coinm_http_error(503, b"busy");
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn contract_error_paths_cover_429_5xx_and_timeout() {
        let transport = SpyTransport::new();
        let adapter =
            BinanceCoinmRestAdapter::new("https://docs.test/binance-coinm", "https://dapi.test");

        transport
            .set_response(
                "https://docs.test/binance-coinm/general-info",
                429,
                "retry_after_ms=999",
            )
            .await;
        let e429 = adapter
            .execute_rest(&transport, "coinm.public.rest.general.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(e429.code, ErrorCode::RateLimited);
        assert_eq!(e429.retry_after_ms, Some(999));

        transport
            .set_response("https://docs.test/binance-coinm/error-code", 502, "oops")
            .await;
        let e5xx = adapter
            .execute_rest(&transport, "coinm.public.rest.errors.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(e5xx.code, ErrorCode::Upstream5xx);

        struct TimeoutTransport;
        impl Transport for TimeoutTransport {
            async fn send_http(
                &self,
                _req: HttpRequest,
                _ctx: RequestContext,
            ) -> Result<HttpResponse, UcelError> {
                Err(UcelError::new(ErrorCode::Timeout, "timeout"))
            }
            async fn connect_ws(
                &self,
                _req: WsConnectRequest,
                _ctx: RequestContext,
            ) -> Result<WsStream, UcelError> {
                Ok(WsStream { connected: true })
            }
        }

        let timeout = TimeoutTransport;
        let e_timeout = adapter
            .execute_rest(&timeout, "coinm.public.rest.common.ref", None, None)
            .await
            .unwrap_err();
        assert_eq!(e_timeout.code, ErrorCode::Timeout);
    }

    #[test]
    fn endpoint_specs_match_catalog_rest_ids_exactly() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let raw =
            std::fs::read_to_string(repo_root.join("docs/exchanges/binance-coinm/catalog.json"))
                .unwrap();
        let catalog: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut impl_ids: Vec<&str> = BinanceCoinmRestAdapter::endpoint_specs()
            .iter()
            .map(|e| e.id)
            .collect();
        let mut catalog_ids: Vec<String> = catalog["rest_endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap().to_string())
            .collect();
        impl_ids.sort_unstable();
        catalog_ids.sort_unstable();
        assert_eq!(
            impl_ids,
            catalog_ids.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    #[test]
    fn coverage_manifest_has_no_rest_gaps() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../coverage/binance-coinm.yaml");
        let raw = std::fs::read_to_string(manifest_path).unwrap();
        let manifest: CoverageManifest = serde_yaml::from_str(&raw).unwrap();
        assert_eq!(manifest.venue, "binance-coinm");
        assert!(manifest.strict);
        for e in &manifest.entries {
            assert!(e.implemented, "id not implemented: {}", e.id);
            assert!(e.tested, "id not tested: {}", e.id);
        }
    }
}
