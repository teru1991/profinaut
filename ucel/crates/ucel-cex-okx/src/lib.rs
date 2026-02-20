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
    pub source_url: String,
    pub path_or_doc: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct OkxRestAdapter {
    endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl OkxRestAdapter {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
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

    pub fn from_specs(specs: Vec<EndpointSpec>) -> Self {
        let mut adapter = Self::new();
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
    ) -> Result<OkxRestResponse, UcelError> {
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
            venue: "okx".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}::{}", spec.source_url, spec.path_or_doc),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_okx_http_error(response.status, &response.body));
        }

        parse_response(&response.body)
    }
}

impl Default for OkxRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OkxEnvelope {
    pub code: String,
    pub msg: String,
    pub data: Vec<OkxDataRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OkxRestResponse {
    Envelope(OkxEnvelope),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OkxDataRecord {
    #[serde(flatten)]
    pub fields: BTreeMap<String, OkxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OkxField {
    Scalar(OkxScalar),
    Object(OkxDataRecord),
    Array(Vec<OkxField>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OkxScalar {
    String(String),
    Number(f64),
    Bool(bool),
    Null(()),
}

#[derive(Debug, Deserialize)]
struct OkxEnvelopeWire {
    code: String,
    #[serde(default)]
    msg: String,
    #[serde(default)]
    data: Vec<OkxDataRecord>,
}

fn parse_response(bytes: &[u8]) -> Result<OkxRestResponse, UcelError> {
    if bytes.is_empty() {
        return Ok(OkxRestResponse::Empty);
    }
    let payload: OkxEnvelopeWire = serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;
    Ok(OkxRestResponse::Envelope(OkxEnvelope {
        code: payload.code,
        msg: payload.msg,
        data: payload.data,
    }))
}

#[derive(Debug, Deserialize)]
struct OkxErrorEnvelope {
    code: Option<String>,
    msg: Option<String>,
    data: Option<Vec<OkxErrorDetail>>,
}

#[derive(Debug, Deserialize)]
struct OkxErrorDetail {
    #[serde(rename = "sCode")]
    s_code: Option<String>,
}

pub fn map_okx_http_error(status: u16, body: &[u8]) -> UcelError {
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

    let envelope = serde_json::from_slice::<OkxErrorEnvelope>(body).ok();
    let code = envelope
        .as_ref()
        .and_then(|v| v.code.as_deref())
        .or_else(|| {
            envelope
                .as_ref()
                .and_then(|v| v.data.as_ref())
                .and_then(|items| items.first())
                .and_then(|d| d.s_code.as_deref())
        })
        .unwrap_or_default();

    let mapped = match code {
        "50011" | "50061" | "50040" => ErrorCode::RateLimited,
        "50113" | "50104" | "50101" => ErrorCode::AuthFailed,
        "50035" | "50036" => ErrorCode::PermissionDenied,
        "51000" | "51008" | "51100" | "51101" | "51131" => ErrorCode::InvalidOrder,
        _ => {
            if status == 401 {
                ErrorCode::AuthFailed
            } else if status == 403 {
                ErrorCode::PermissionDenied
            } else if status == 400 || status == 404 || status == 409 || status == 422 {
                ErrorCode::InvalidOrder
            } else {
                ErrorCode::Network
            }
        }
    };

    let message = envelope
        .and_then(|v| v.msg)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| format!("okx http error status={status} code={code}"));
    UcelError::new(mapped, message)
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    visibility: String,
    path_or_doc: String,
    source_url: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/okx/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid okx catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            source_url: entry.source_url,
            path_or_doc: entry.path_or_doc,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
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
        let adapter = OkxRestAdapter::new();
        assert_eq!(adapter.endpoint_specs().len(), 4);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn parses_success_payload() {
        let adapter = OkxRestAdapter::from_specs(vec![EndpointSpec {
            id: "x".into(),
            method: "GET".into(),
            source_url: "https://www.okx.com/docs-v5/en/".into(),
            path_or_doc: "doc-ref".into(),
            requires_auth: false,
        }]);
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"{"code":"0","msg":"","data":[{"instId":"BTC-USDT"}]}"#),
        }));
        let result = adapter
            .execute_rest(&transport, "x", None, None)
            .await
            .unwrap();
        assert!(matches!(result, OkxRestResponse::Envelope(_)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_and_5xx_and_timeout() {
        let rate_limited = map_okx_http_error(429, b"retry_after_ms=1700");
        assert_eq!(rate_limited.code, ErrorCode::RateLimited);
        assert_eq!(rate_limited.retry_after_ms, Some(1700));

        let upstream = map_okx_http_error(503, br#"{}"#);
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        let transport = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "t")));
        let adapter = OkxRestAdapter::from_specs(vec![EndpointSpec {
            id: "x".into(),
            method: "GET".into(),
            source_url: "https://www.okx.com/docs-v5/en/".into(),
            path_or_doc: "doc-ref".into(),
            requires_auth: false,
        }]);
        let err = adapter
            .execute_rest(&transport, "x", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);
    }

    #[test]
    fn maps_code_based_errors() {
        let auth = map_okx_http_error(401, br#"{"code":"50113","msg":"x"}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);

        let perm = map_okx_http_error(403, br#"{"code":"50035","msg":"x"}"#);
        assert_eq!(perm.code, ErrorCode::PermissionDenied);

        let invalid = map_okx_http_error(400, br#"{"code":"51000","msg":"x"}"#);
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_transport_hit() {
        let adapter = OkxRestAdapter::from_specs(vec![EndpointSpec {
            id: "private".into(),
            method: "POST".into(),
            source_url: "https://www.okx.com/docs-v5/en/".into(),
            path_or_doc: "doc-family".into(),
            requires_auth: true,
        }]);
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
        let adapter = OkxRestAdapter::new();
        for spec in adapter.endpoint_specs() {
            let transport = SpyTransport::with_response(Ok(HttpResponse {
                status: 200,
                body: Bytes::from_static(br#"{"code":"0","msg":"","data":[]}"#),
            }));
            let out = adapter
                .execute_rest(&transport, &spec.id, None, Some("k".into()))
                .await;
            assert!(out.is_ok(), "id={} should parse fixture", spec.id);
        }
    }
}
