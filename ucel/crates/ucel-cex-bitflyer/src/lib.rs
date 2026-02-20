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
    pub response_shape: String,
}

#[derive(Debug, Clone)]
pub struct BitflyerRestAdapter {
    pub base_url: Arc<str>,
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BitflyerRestAdapter {
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
    ) -> Result<BitflyerRestResponse, UcelError> {
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
            venue: "bitflyer".into(),
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
            return Err(map_bitflyer_http_error(response.status, &response.body));
        }

        parse_response_for_shape(&response.body, &spec.response_shape)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitflyerRestResponse {
    Object(BitflyerObject),
    ArrayObject(Vec<BitflyerObject>),
    ArrayString(Vec<String>),
    Text(String),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BitflyerObject {
    #[serde(flatten)]
    pub fields: BTreeMap<String, BitflyerField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitflyerField {
    Scalar(BitflyerScalar),
    Object(BitflyerObject),
    Array(Vec<BitflyerField>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitflyerScalar {
    String(String),
    Number(f64),
    Bool(bool),
    Null(()),
}

fn parse_response_for_shape(bytes: &[u8], shape: &str) -> Result<BitflyerRestResponse, UcelError> {
    if bytes.is_empty() {
        return Ok(BitflyerRestResponse::Empty);
    }

    match shape {
        "object" => serde_json::from_slice::<BitflyerObject>(bytes)
            .map(BitflyerRestResponse::Object)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "array<object>" => serde_json::from_slice::<Vec<BitflyerObject>>(bytes)
            .map(BitflyerRestResponse::ArrayObject)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "array<string>" => serde_json::from_slice::<Vec<String>>(bytes)
            .map(BitflyerRestResponse::ArrayString)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}"))),
        "text/spec" => std::str::from_utf8(bytes)
            .map(|s| BitflyerRestResponse::Text(s.to_string()))
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("text parse error: {e}"))),
        _ => Err(UcelError::new(
            ErrorCode::Internal,
            format!("unsupported response shape: {shape}"),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct BitflyerErrorEnvelope {
    status: Option<i64>,
    code: Option<String>,
    error_code: Option<String>,
}

pub fn map_bitflyer_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = extract_retry_after_ms(body);
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let envelope = serde_json::from_slice::<BitflyerErrorEnvelope>(body).ok();
    let code_field = envelope
        .as_ref()
        .and_then(|e| e.error_code.as_deref().or(e.code.as_deref()))
        .unwrap_or_default()
        .to_ascii_uppercase();
    let status_field = envelope
        .as_ref()
        .and_then(|e| e.status)
        .unwrap_or(status as i64);

    let code = if status == 401 || status_field == -200 || code_field == "AUTH_ERROR" {
        ErrorCode::AuthFailed
    } else if status == 403 || code_field == "PERMISSION_DENIED" {
        ErrorCode::PermissionDenied
    } else if status == 400 || status == 404 || status == 409 || status == 422 {
        ErrorCode::InvalidOrder
    } else {
        ErrorCode::Network
    };

    UcelError::new(code, format!("bitflyer http error status={status}"))
}

fn extract_retry_after_ms(body: &[u8]) -> Option<u64> {
    if let Ok(v) = serde_json::from_slice::<BTreeMap<String, String>>(body) {
        if let Some(raw) = v.get("retry_after_ms").or_else(|| v.get("retry-after-ms")) {
            if let Ok(parsed) = raw.parse::<u64>() {
                return Some(parsed);
            }
        }
        if let Some(raw) = v.get("retry_after") {
            if let Ok(sec) = raw.parse::<u64>() {
                return Some(sec.saturating_mul(1000));
            }
        }
    }

    std::str::from_utf8(body)
        .ok()
        .and_then(|b| b.split("retry_after_ms=").nth(1))
        .and_then(|s| s.trim().parse::<u64>().ok())
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
    access: Option<String>,
    auth: Option<AuthSpec>,
    response: ResponseSpec,
}

#[derive(Debug, Deserialize)]
struct AuthSpec {
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Debug, Deserialize)]
struct ResponseSpec {
    shape: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/bitflyer/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bitflyer catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            path: entry.path,
            requires_auth: entry
                .access
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case("private"))
                .unwrap_or(false)
                || entry
                    .auth
                    .as_ref()
                    .map(|a| !a.auth_type.eq_ignore_ascii_case("none"))
                    .unwrap_or(false),
            response_shape: entry.response.shape,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        calls: AtomicUsize,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
        last_ctx: Mutex<Option<RequestContext>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                response: Mutex::new(Some(resp)),
                last_ctx: Mutex::new(None),
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
            *self.last_ctx.lock().unwrap() = Some(ctx);
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
        let adapter = BitflyerRestAdapter::new("http://localhost");
        assert_eq!(adapter.endpoint_specs().len(), 49);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn parses_shape_specific_payloads() {
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![
                EndpointSpec {
                    id: "obj".into(),
                    method: "GET".into(),
                    path: "/obj".into(),
                    requires_auth: false,
                    response_shape: "object".into(),
                },
                EndpointSpec {
                    id: "arr_obj".into(),
                    method: "GET".into(),
                    path: "/arr_obj".into(),
                    requires_auth: false,
                    response_shape: "array<object>".into(),
                },
                EndpointSpec {
                    id: "arr_str".into(),
                    method: "GET".into(),
                    path: "/arr_str".into(),
                    requires_auth: false,
                    response_shape: "array<string>".into(),
                },
                EndpointSpec {
                    id: "txt".into(),
                    method: "GET".into(),
                    path: "/txt".into(),
                    requires_auth: false,
                    response_shape: "text/spec".into(),
                },
            ],
        );

        let ok = [
            ("obj", Bytes::from_static(br#"{"x":1}"#)),
            ("arr_obj", Bytes::from_static(br#"[{"x":1}]"#)),
            ("arr_str", Bytes::from_static(br#"["trade","withdraw"]"#)),
            ("txt", Bytes::from_static(b"any text")),
        ];

        for (id, body) in ok {
            let transport = SpyTransport::with_response(Ok(HttpResponse { status: 200, body }));
            let out = adapter
                .execute_rest(&transport, id, None, None)
                .await
                .unwrap();
            assert!(!matches!(out, BitflyerRestResponse::Empty));
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_and_5xx_and_timeout() {
        let rate_limited = map_bitflyer_http_error(429, br#"{"retry_after":"2"}"#);
        assert_eq!(rate_limited.code, ErrorCode::RateLimited);
        assert_eq!(rate_limited.retry_after_ms, Some(2000));

        let upstream = map_bitflyer_http_error(503, br#"{}"#);
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        let transport = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "t")));
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "x".into(),
                method: "GET".into(),
                path: "/x".into(),
                requires_auth: false,
                response_shape: "object".into(),
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
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "private".into(),
                method: "POST".into(),
                path: "/p".into(),
                requires_auth: true,
                response_shape: "object".into(),
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
    async fn public_endpoint_has_zero_key_path() {
        let adapter = BitflyerRestAdapter::from_specs(
            "http://localhost",
            vec![EndpointSpec {
                id: "public".into(),
                method: "GET".into(),
                path: "/public".into(),
                requires_auth: false,
                response_shape: "object".into(),
            }],
        );
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"{}"#),
        }));

        let _ = adapter
            .execute_rest(&transport, "public", None, None)
            .await
            .unwrap();
        let ctx = transport.last_ctx.lock().unwrap().clone().unwrap();
        assert!(ctx.key_id.is_none());
        assert!(!ctx.requires_auth);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn contract_test_all_rest_ids_can_parse_minimal_fixture() {
        let adapter = BitflyerRestAdapter::new("http://localhost");

        for spec in adapter.endpoint_specs() {
            let fixture = match spec.response_shape.as_str() {
                "object" => Bytes::from_static(br#"{}"#),
                "array<object>" => Bytes::from_static(br#"[]"#),
                "array<string>" => Bytes::from_static(br#"[]"#),
                "text/spec" => Bytes::from_static(b"spec"),
                _ => panic!("unexpected shape for {}", spec.id),
            };
            let transport = SpyTransport::with_response(Ok(HttpResponse {
                status: 200,
                body: fixture,
            }));
            let key = spec.requires_auth.then(|| "k".to_string());
            let out = adapter.execute_rest(&transport, &spec.id, None, key).await;
            assert!(out.is_ok(), "id={} should parse fixture", spec.id);
        }
    }

    #[test]
    fn maps_auth_and_permission_and_invalid_order() {
        let auth = map_bitflyer_http_error(401, br#"{"status":-200}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let perm = map_bitflyer_http_error(403, br#"{"error_code":"PERMISSION_DENIED"}"#);
        assert_eq!(perm.code, ErrorCode::PermissionDenied);

        let invalid = map_bitflyer_http_error(400, br#"{"status":-1}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);
    }
}
