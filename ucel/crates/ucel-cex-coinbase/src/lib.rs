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
    pub transport_enabled: bool,
}

const ENDPOINTS: [EndpointSpec; 7] = [
    EndpointSpec {
        id: "advanced.crypto.public.rest.reference.introduction",
        method: "GET",
        base_url: "https://api.coinbase.com",
        path: "/api/v3/brokerage/*",
        requires_auth: false,
        transport_enabled: true,
    },
    EndpointSpec {
        id: "advanced.crypto.private.rest.reference.introduction",
        method: "GET",
        base_url: "https://api.coinbase.com",
        path: "/api/v3/brokerage/*",
        requires_auth: true,
        transport_enabled: true,
    },
    EndpointSpec {
        id: "exchange.crypto.public.rest.reference.introduction",
        method: "GET",
        base_url: "https://api.exchange.coinbase.com",
        path: "/*",
        requires_auth: false,
        transport_enabled: true,
    },
    EndpointSpec {
        id: "exchange.crypto.private.rest.reference.introduction",
        method: "GET",
        base_url: "https://api.exchange.coinbase.com",
        path: "/*",
        requires_auth: true,
        transport_enabled: true,
    },
    EndpointSpec {
        id: "intx.crypto.public.rest.reference.welcome",
        method: "GET",
        base_url: "not_applicable",
        path: "not_applicable",
        requires_auth: false,
        transport_enabled: false,
    },
    EndpointSpec {
        id: "intx.crypto.private.rest.reference.welcome",
        method: "GET",
        base_url: "not_applicable",
        path: "not_applicable",
        requires_auth: true,
        transport_enabled: false,
    },
    EndpointSpec {
        id: "other.other.public.rest.docs.root",
        method: "GET",
        base_url: "https://docs.cdp.coinbase.com",
        path: "/",
        requires_auth: false,
        transport_enabled: true,
    },
];

#[derive(Clone)]
pub struct CoinbaseRestAdapter {
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
    default_policy_id: Arc<str>,
}

impl CoinbaseRestAdapter {
    pub fn new() -> Self {
        Self {
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
            default_policy_id: Arc::from("default"),
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
    ) -> Result<CoinbaseRestResponse, UcelError> {
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
            venue: "coinbase".into(),
            policy_id: self.default_policy_id.to_string(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        if !spec.transport_enabled {
            return Ok(CoinbaseRestResponse::ReferenceOnly(
                CoinbaseReferenceResponse {
                    id: spec.id.to_string(),
                    source: "discoverability-only".to_string(),
                },
            ));
        }

        let req = HttpRequest {
            method: spec.method.to_string(),
            path: format!("{}{}", spec.base_url, spec.path),
            body,
        };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_coinbase_http_error(response.status, &response.body));
        }

        Ok(CoinbaseRestResponse::Reference(parse_json(&response.body)?))
    }
}

impl Default for CoinbaseRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoinbaseReferenceResponse {
    pub id: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinbaseRestResponse {
    Reference(CoinbaseReferenceResponse),
    ReferenceOnly(CoinbaseReferenceResponse),
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct CoinbaseErrorEnvelope {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    error_type: Option<String>,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    retry_after_ms: Option<u64>,
}

pub fn map_coinbase_http_error(status: u16, body: &[u8]) -> UcelError {
    let parsed = serde_json::from_slice::<CoinbaseErrorEnvelope>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|e| e.code.as_deref())
        .unwrap_or_default();
    let error_type = parsed
        .as_ref()
        .and_then(|e| e.error_type.as_deref())
        .unwrap_or_default();
    let field = parsed
        .as_ref()
        .and_then(|e| e.field.as_deref())
        .unwrap_or_default();

    if status == 429 {
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited by coinbase");
        err.retry_after_ms = parsed.and_then(|e| e.retry_after_ms);
        err.ban_risk = true;
        return err;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "coinbase upstream server error");
    }

    if status == 408 {
        return UcelError::new(ErrorCode::Timeout, "coinbase request timeout");
    }

    if status == 401 || code == "invalid_api_key" || error_type == "authentication_error" {
        return UcelError::new(ErrorCode::AuthFailed, "coinbase authentication failed");
    }

    if status == 403 || code == "permission_denied" {
        return UcelError::new(ErrorCode::PermissionDenied, "coinbase permission denied");
    }

    if status == 400 && (code == "invalid_order" || field == "order") {
        return UcelError::new(ErrorCode::InvalidOrder, "coinbase order validation failed");
    }

    UcelError::new(
        ErrorCode::Internal,
        format!("unmapped coinbase error status={status} code={code} type={error_type}"),
    )
}

pub struct CoinbaseExchange;

impl Exchange for CoinbaseExchange {
    fn name(&self) -> &'static str {
        "coinbase"
    }

    fn execute(&self, _op: OpName) -> Result<(), UcelError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use ucel_testkit::{evaluate_coverage_gate, load_coverage_manifest};
    use ucel_transport::{HttpResponse, Transport, WsConnectRequest, WsStream};

    #[derive(Clone, Default)]
    struct SpyTransport {
        calls: Arc<Mutex<Vec<RequestContext>>>,
        response: Arc<Mutex<Option<Result<HttpResponse, UcelError>>>>,
    }

    impl SpyTransport {
        fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(
            &self,
            _req: HttpRequest,
            ctx: RequestContext,
        ) -> Result<HttpResponse, UcelError> {
            self.calls.lock().unwrap().push(ctx);
            self.response.lock().unwrap().take().unwrap_or_else(|| {
                Ok(HttpResponse {
                    status: 200,
                    body: Bytes::from_static(br#"{"id":"ok","source":"fixture"}"#),
                })
            })
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
    async fn private_endpoint_rejects_without_auth_before_transport() {
        let adapter = CoinbaseRestAdapter::new();
        let transport = SpyTransport::default();
        let err = adapter
            .execute_rest(
                &transport,
                "advanced.crypto.private.rest.reference.introduction",
                None,
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.call_count(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_endpoint_does_not_require_key_path() {
        let adapter = CoinbaseRestAdapter::new();
        let transport = SpyTransport::default();
        let _ = adapter
            .execute_rest(
                &transport,
                "advanced.crypto.public.rest.reference.introduction",
                None,
                None,
            )
            .await
            .unwrap();
        let calls = transport.calls.lock().unwrap();
        assert_eq!(calls[0].key_id, None);
    }

    #[test]
    fn error_mapping_uses_code_and_field() {
        let invalid = map_coinbase_http_error(
            400,
            br#"{"code":"invalid_order","field":"order","error_type":"validation_error"}"#,
        );
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);

        let denied = map_coinbase_http_error(403, br#"{"code":"permission_denied"}"#);
        assert_eq!(denied.code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn coverage_manifest_is_strict_and_has_no_gaps() {
        let manifest_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/coinbase.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert!(manifest.strict);
        let gaps = evaluate_coverage_gate(&manifest);
        assert!(gaps.is_empty(), "strict coverage gate found gaps: {gaps:?}");
    }
}
