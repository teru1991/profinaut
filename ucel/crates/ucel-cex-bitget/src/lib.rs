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
    pub base_url: String,
    pub path: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct BitgetRestAdapter {
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl Default for BitgetRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BitgetRestAdapter {
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

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<BitgetRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|entry| entry.id == endpoint_id)
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
            venue: "bitget".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.clone(),
            path: format!("{}{}", spec.base_url, spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_bitget_http_error(response.status, &response.body));
        }

        parse_response(&response.body)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitgetRestResponse {
    pub fields: BTreeMap<String, BitgetField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BitgetField {
    String(String),
    Number(f64),
    Bool(bool),
    Object(BTreeMap<String, BitgetField>),
    Array(Vec<BitgetField>),
    Null,
}

fn parse_response(bytes: &[u8]) -> Result<BitgetRestResponse, UcelError> {
    let fields = serde_json::from_slice::<BTreeMap<String, BitgetField>>(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;
    Ok(BitgetRestResponse { fields })
}

#[derive(Debug, Deserialize)]
struct BitgetErrorEnvelope {
    code: Option<String>,
}

pub fn map_bitget_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|text| text.split("retry_after_ms=").nth(1))
            .and_then(|value| value.trim().parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        err.ban_risk = true;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let envelope = serde_json::from_slice::<BitgetErrorEnvelope>(body).ok();
    let code = envelope
        .as_ref()
        .and_then(|err| err.code.as_deref())
        .unwrap_or_default();

    let mapped = match (status, code) {
        (401, _) | (_, "40014") | (_, "40017") => ErrorCode::AuthFailed,
        (403, _) | (_, "40023") => ErrorCode::PermissionDenied,
        (400, _) | (404, _) | (409, _) | (422, _) | (_, "40020") => ErrorCode::InvalidOrder,
        _ => ErrorCode::Network,
    };

    UcelError::new(
        mapped,
        format!("bitget http error status={status} code={code}"),
    )
}

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    method: String,
    base_url: String,
    path: String,
    visibility: String,
    requires_auth: Option<bool>,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/bitget/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid bitget catalog");

    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            base_url: entry.base_url,
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
    use std::sync::Mutex;
    use ucel_transport::{HttpResponse, WsConnectRequest, WsStream};

    #[derive(Default)]
    struct SpyTransport {
        send_http_calls: Mutex<Vec<RequestContext>>,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                send_http_calls: Mutex::new(Vec::new()),
                response: Mutex::new(Some(resp)),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.send_http_calls.lock().unwrap().push(ctx);
            self.response.lock().unwrap().take().unwrap()
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream { connected: true })
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn catalog_rest_rows_are_typed() {
        let adapter = BitgetRestAdapter::new();
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"{"unknown":{"k":"v"}}"#),
        }));

        for endpoint in adapter.endpoints.iter() {
            let response = adapter
                .execute_rest(&transport, &endpoint.id, None, Some("key-ignored".into()))
                .await
                .unwrap();
            assert!(response.fields.contains_key("unknown"));
        }

        let calls = transport.send_http_calls.lock().unwrap();
        assert_eq!(calls.len(), adapter.endpoints.len());
        assert!(calls.iter().all(|ctx| !ctx.requires_auth));
        assert!(calls.iter().all(|ctx| ctx.key_id.is_none()));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_preflight_rejects_without_transport() {
        let adapter = BitgetRestAdapter::from_specs(vec![EndpointSpec {
            id: "bitget.private.rest.synthetic".into(),
            method: "GET".into(),
            base_url: "https://api.bitget.com".into(),
            path: "/api/v2/private".into(),
            requires_auth: true,
        }]);
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(b"{}"),
        }));

        let err = adapter
            .execute_rest(&transport, "bitget.private.rest.synthetic", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert!(transport.send_http_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn error_mapping_handles_429_5xx_and_code_based_auth() {
        let rate_limited = map_bitget_http_error(429, b"retry_after_ms=1200");
        assert_eq!(rate_limited.code, ErrorCode::RateLimited);
        assert_eq!(rate_limited.retry_after_ms, Some(1200));

        let upstream = map_bitget_http_error(503, b"{}");
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        let auth = map_bitget_http_error(400, br#"{"code":"40014"}"#);
        assert_eq!(auth.code, ErrorCode::AuthFailed);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn timeout_from_transport_is_propagated() {
        let adapter = BitgetRestAdapter::new();
        let transport =
            SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "timeout")));
        let endpoint_id = adapter.endpoints[0].id.clone();

        let err = adapter
            .execute_rest(&transport, &endpoint_id, None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);
    }

    #[test]
    fn strict_coverage_has_no_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitget.yaml");
        #[derive(serde::Deserialize)]
        struct CoverageEntry {
            implemented: bool,
            tested: bool,
        }
        #[derive(serde::Deserialize)]
        struct CoverageManifest {
            strict: bool,
            entries: Vec<CoverageEntry>,
        }

        let raw = std::fs::read_to_string(manifest_path).unwrap();
        let manifest: CoverageManifest = serde_yaml::from_str(&raw).unwrap();
        assert!(manifest.strict);
        assert!(manifest.entries.iter().all(|e| e.implemented && e.tested));
    }
}

pub mod symbols;
pub mod ws;
pub mod ws_manager;
pub mod channels;
