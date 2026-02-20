use bytes::Bytes;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub path: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct BitmexRestAdapter {
    pub base_url: Arc<str>,
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BitmexRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            base_url: Arc::from(base_url.into()),
            endpoints: Arc::new(load_endpoint_specs()),
            http_client,
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn from_specs(base_url: impl Into<String>, specs: Vec<EndpointSpec>) -> Self {
        let mut adapter = Self::new(base_url);
        adapter.endpoints = Arc::new(specs);
        adapter
    }

    pub fn endpoint_specs(&self) -> &[EndpointSpec] {
        &self.endpoints
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<BitmexRestResponse, UcelError> {
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
            op: OpName::FetchStatus,
            venue: "bitmex".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}{}", self.base_url, spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_bitmex_http_error(response.status, &response.body));
        }

        parse_response(&response.body)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitmexRestResponse {
    Object(BitmexObject),
    Array(Vec<BitmexObject>),
    Scalar(BitmexScalar),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BitmexObject {
    #[serde(flatten)]
    pub fields: BTreeMap<String, BitmexField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitmexField {
    Scalar(BitmexScalar),
    Object(BitmexObject),
    Array(Vec<BitmexField>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitmexScalar {
    String(String),
    Number(f64),
    Bool(bool),
    Null(()),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ParsedPayload {
    Object(BitmexObject),
    Array(Vec<BitmexObject>),
    Scalar(BitmexScalar),
}

fn parse_response(bytes: &[u8]) -> Result<BitmexRestResponse, UcelError> {
    if bytes.is_empty() {
        return Ok(BitmexRestResponse::Empty);
    }
    let payload: ParsedPayload = serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;
    Ok(match payload {
        ParsedPayload::Object(v) => BitmexRestResponse::Object(v),
        ParsedPayload::Array(v) => BitmexRestResponse::Array(v),
        ParsedPayload::Scalar(v) => BitmexRestResponse::Scalar(v),
    })
}

#[derive(Debug, Deserialize)]
struct BitmexErrorEnvelope {
    error: Option<BitmexErrorBody>,
}

#[derive(Debug, Deserialize)]
struct BitmexErrorBody {
    name: Option<String>,
}

pub fn map_bitmex_http_error(status: u16, body: &[u8]) -> UcelError {
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

    let envelope = serde_json::from_slice::<BitmexErrorEnvelope>(body).ok();
    let name = envelope
        .as_ref()
        .and_then(|v| v.error.as_ref())
        .and_then(|e| e.name.as_deref())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let code = if status == 401 || name.contains("auth") {
        ErrorCode::AuthFailed
    } else if status == 403 || name.contains("permission") {
        ErrorCode::PermissionDenied
    } else if status == 400 || status == 404 || status == 409 || status == 422 {
        ErrorCode::InvalidOrder
    } else {
        ErrorCode::Network
    };

    UcelError::new(code, format!("bitmex http error status={status}"))
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    path: String,
    visibility: String,
    requires_auth: Option<bool>,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/bitmex/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bitmex catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            path: entry.path,
            requires_auth: entry
                .requires_auth
                .unwrap_or_else(|| entry.visibility.eq_ignore_ascii_case("private")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use ucel_core::ErrorCode;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        calls: AtomicUsize,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                response: Mutex::new(Some(resp)),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            _ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
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

    #[test]
    fn loads_all_rest_rows_from_catalog() {
        let adapter = BitmexRestAdapter::new("http://localhost");
        assert_eq!(adapter.endpoint_specs().len(), 95);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn parses_success_payload() {
        let adapter = BitmexRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "x".into(),
                method: "GET".into(),
                path: "/x".into(),
                requires_auth: false,
            }],
        );
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"[{"a":1}]"#),
        }));
        let result = adapter
            .execute_rest(&transport, "x", None, None)
            .await
            .unwrap();
        assert!(matches!(result, BitmexRestResponse::Array(_)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_and_5xx_and_timeout() {
        let rate_limited = map_bitmex_http_error(429, b"retry_after_ms=1200");
        assert_eq!(rate_limited.code, ErrorCode::RateLimited);
        assert_eq!(rate_limited.retry_after_ms, Some(1200));

        let upstream = map_bitmex_http_error(503, br#"{}"#);
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        let transport = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "t")));
        let adapter = BitmexRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "x".into(),
                method: "GET".into(),
                path: "/x".into(),
                requires_auth: false,
            }],
        );
        let err = adapter
            .execute_rest(&transport, "x", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_transport_hit() {
        let adapter = BitmexRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "private".into(),
                method: "POST".into(),
                path: "/p".into(),
                requires_auth: true,
            }],
        );
        let transport = SpyTransport::default();
        let err = adapter
            .execute_rest(&transport, "private", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn contract_test_all_rest_ids_can_parse_minimal_fixture() {
        let adapter = BitmexRestAdapter::new("http://localhost");
        for spec in adapter.endpoint_specs() {
            let transport = SpyTransport::with_response(Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(br#"{}"#),
            }));
            let out = adapter
                .execute_rest(&transport, &spec.id, None, Some("k".into()))
                .await;
            assert!(out.is_ok(), "id={} should parse fixture", spec.id);
        }
    }
}
