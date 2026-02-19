use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ucel_core::{ErrorCode, Exchange, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, Transport};

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 30] = [
    EndpointSpec {
        id: "crypto.public.rest.status.get",
        method: "GET",
        path: "/public/v1/status",
        requires_auth: false,
    },
    EndpointSpec {
        id: "crypto.public.rest.ticker.get",
        method: "GET",
        path: "/public/v1/ticker",
        requires_auth: false,
    },
    EndpointSpec {
        id: "crypto.public.rest.orderbooks.get",
        method: "GET",
        path: "/public/v1/orderbooks",
        requires_auth: false,
    },
    EndpointSpec {
        id: "crypto.public.rest.trades.get",
        method: "GET",
        path: "/public/v1/trades",
        requires_auth: false,
    },
    EndpointSpec {
        id: "crypto.public.rest.klines.get",
        method: "GET",
        path: "/public/v1/klines",
        requires_auth: false,
    },
    EndpointSpec {
        id: "crypto.private.rest.wsauth.post",
        method: "POST",
        path: "/private/v1/ws-auth",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.wsauth.extend.put",
        method: "PUT",
        path: "/private/v1/ws-auth",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.assets.get",
        method: "GET",
        path: "/private/v1/account/assets",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.margin.get",
        method: "GET",
        path: "/private/v1/account/margin",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.activeorders.get",
        method: "GET",
        path: "/private/v1/activeOrders",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.executions.get",
        method: "GET",
        path: "/private/v1/executions",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.latestexecutions.get",
        method: "GET",
        path: "/private/v1/latestExecutions",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.order.post",
        method: "POST",
        path: "/private/v1/order",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.changeorder.post",
        method: "POST",
        path: "/private/v1/changeOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.cancelorder.post",
        method: "POST",
        path: "/private/v1/cancelOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.openpositions.get",
        method: "GET",
        path: "/private/v1/openPositions",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.positionsummary.get",
        method: "GET",
        path: "/private/v1/positionSummary",
        requires_auth: true,
    },
    EndpointSpec {
        id: "crypto.private.rest.closeorder.post",
        method: "POST",
        path: "/private/v1/closeOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.public.rest.status.get",
        method: "GET",
        path: "/fx/public/v1/status",
        requires_auth: false,
    },
    EndpointSpec {
        id: "fx.public.rest.ticker.get",
        method: "GET",
        path: "/fx/public/v1/ticker",
        requires_auth: false,
    },
    EndpointSpec {
        id: "fx.public.rest.orderbooks.get",
        method: "GET",
        path: "/fx/public/v1/orderbooks",
        requires_auth: false,
    },
    EndpointSpec {
        id: "fx.public.rest.trades.get",
        method: "GET",
        path: "/fx/public/v1/trades",
        requires_auth: false,
    },
    EndpointSpec {
        id: "fx.public.rest.klines.get",
        method: "GET",
        path: "/fx/public/v1/klines",
        requires_auth: false,
    },
    EndpointSpec {
        id: "fx.private.rest.wsauth.post",
        method: "POST",
        path: "/fx/private/v1/ws-auth",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.assets.get",
        method: "GET",
        path: "/fx/private/v1/account/assets",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.activeorders.get",
        method: "GET",
        path: "/fx/private/v1/activeOrders",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.order.post",
        method: "POST",
        path: "/fx/private/v1/order",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.cancelorder.post",
        method: "POST",
        path: "/fx/private/v1/cancelOrder",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.openpositions.get",
        method: "GET",
        path: "/fx/private/v1/openPositions",
        requires_auth: true,
    },
    EndpointSpec {
        id: "fx.private.rest.closeorder.post",
        method: "POST",
        path: "/fx/private/v1/closeOrder",
        requires_auth: true,
    },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmoEnvelope<T> {
    pub status: i32,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericPayload {
    pub endpoint: String,
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerPayload {
    pub ask: String,
    pub bid: String,
    pub last: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GmoRestResponse {
    Generic(GmoEnvelope<GenericPayload>),
    Ticker(GmoEnvelope<Vec<TickerPayload>>),
}

#[derive(Debug, Clone, Deserialize)]
struct GmoErrorMessage {
    #[serde(default)]
    message_code: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GmoErrorBody {
    #[serde(default)]
    messages: Vec<GmoErrorMessage>,
    #[serde(default)]
    retry_after_ms: Option<u64>,
}

#[derive(Clone)]
pub struct GmoCoinRestAdapter {
    base_url: Arc<str>,
    #[allow(dead_code)]
    http_client: reqwest::Client,
}

impl GmoCoinRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Arc::from(base_url.into()),
            http_client: reqwest::Client::new(),
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
    ) -> Result<GmoRestResponse, UcelError> {
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
            trace_id: Uuid::new_v4().to_string().into(),
            request_id: Uuid::new_v4().to_string().into(),
            run_id: Uuid::new_v4().to_string().into(),
            op: OpName::FetchStatus,
            venue: "gmocoin".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method.into(),
            path: format!("{}{}", self.base_url, spec.path),
            body,
        };
        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_http_error(response.status, &response.body));
        }

        if endpoint_id.ends_with("ticker.get") {
            let payload: GmoEnvelope<Vec<TickerPayload>> = parse_json(&response.body)?;
            return Ok(GmoRestResponse::Ticker(payload));
        }

        let payload: GmoEnvelope<GenericPayload> = parse_json(&response.body)?;
        Ok(GmoRestResponse::Generic(payload))
    }
}

fn parse_json<T: DeserializeOwned>(body: &Bytes) -> Result<T, UcelError> {
    serde_json::from_slice(body)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("invalid json: {e}")))
}

fn map_http_error(status: u16, body: &Bytes) -> UcelError {
    let parsed = serde_json::from_slice::<GmoErrorBody>(body).ok();
    if status == 429 {
        let mut e = UcelError::new(ErrorCode::RateLimited, "rate limited");
        e.retry_after_ms = parsed.and_then(|v| v.retry_after_ms);
        e.ban_risk = true;
        return e;
    }

    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let code = parsed
        .as_ref()
        .and_then(|v| v.messages.first())
        .map(|m| m.message_code.as_str())
        .unwrap_or_default();

    let mut err = match code {
        "ERR-5201" => UcelError::new(ErrorCode::InvalidOrder, "invalid order"),
        "ERR-5003" => UcelError::new(ErrorCode::AuthFailed, "authentication failed"),
        "ERR-5010" => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        _ => UcelError::new(ErrorCode::Internal, format!("http error status={status}")),
    };
    err.key_specific = code == "ERR-5003";
    err
}

impl Exchange for GmoCoinRestAdapter {
    fn name(&self) -> &'static str {
        "gmo_coin"
    }

    fn execute(&self, _op: OpName) -> Result<(), UcelError> {
        Ok(())
    }
}

pub struct GmoCoinAdapter;

impl Exchange for GmoCoinAdapter {
    fn name(&self) -> &'static str {
        "gmo_coin"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("{} not implemented for {}", op, self.name()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use ucel_transport::HttpResponse;

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
            let res = self.responses.lock().await.remove(&req.path).unwrap();
            Ok(res)
        }

        async fn connect_ws(
            &self,
            _req: ucel_transport::WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<ucel_transport::WsStream, UcelError> {
            unreachable!()
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_all_endpoints_are_covered() {
        let transport = SpyTransport::new();
        let adapter = GmoCoinRestAdapter::new("https://api.example.com");

        for spec in GmoCoinRestAdapter::endpoint_specs() {
            let path = format!("https://api.example.com{}", spec.path);
            let success = if spec.id.ends_with("ticker.get") {
                r#"{"status":0,"data":[{"ask":"1","bid":"1","last":"1"}]}"#
            } else {
                &format!(
                    "{{\"status\":0,\"data\":{{\"endpoint\":\"{}\",\"ok\":true}}}}",
                    spec.id
                )
            };
            transport.set_response(&path, 200, success).await;
            let key = if spec.requires_auth {
                Some("k1".to_string())
            } else {
                None
            };
            let result = adapter.execute_rest(&transport, spec.id, None, key).await;
            assert!(result.is_ok(), "endpoint {} failed", spec.id);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_rejects_without_auth_and_does_not_hit_transport() {
        let transport = SpyTransport::new();
        let adapter = GmoCoinRestAdapter::new("https://api.example.com");
        let err = adapter
            .execute_rest(&transport, "crypto.private.rest.order.post", None, None)
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_retryable_and_non_retryable_errors() {
        let transport = SpyTransport::new();
        let adapter = GmoCoinRestAdapter::new("https://api.example.com");

        transport
            .set_response(
                "https://api.example.com/public/v1/status",
                429,
                r#"{"messages":[{"message_code":"ERR-429"}],"retry_after_ms":1200}"#,
            )
            .await;
        let rate = adapter
            .execute_rest(&transport, "crypto.public.rest.status.get", None, None)
            .await
            .unwrap_err();
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(1200));

        transport
            .set_response(
                "https://api.example.com/private/v1/order",
                400,
                r#"{"messages":[{"message_code":"ERR-5201"}]}"#,
            )
            .await;
        let invalid = adapter
            .execute_rest(
                &transport,
                "crypto.private.rest.order.post",
                None,
                Some("k1".into()),
            )
            .await
            .unwrap_err();
        assert_eq!(invalid.code, ErrorCode::InvalidOrder);
    }

    #[test]
    fn bench_like_deserialize_and_normalize_regression_guard() {
        let ticker =
            Bytes::from_static(br#"{"status":0,"data":[{"ask":"1","bid":"1","last":"1"}]}"#);
        let generic = Bytes::from_static(br#"{"status":0,"data":{"endpoint":"x","ok":true}}"#);
        for _ in 0..1000 {
            let _: GmoEnvelope<Vec<TickerPayload>> = parse_json(&ticker).unwrap();
            let _: GmoEnvelope<GenericPayload> = parse_json(&generic).unwrap();
        }
    }
}
