use serde::de::DeserializeOwned;
use serde::Deserialize;
use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    pub rest_endpoints: Vec<RestEndpoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestEndpoint {
    pub id: String,
    pub visibility: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct RestRequest {
    pub method: String,
    pub url: String,
    pub auth: Option<AuthContext>,
}

#[derive(Debug, Clone)]
pub struct RestResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after_ms: Option<u64>,
}

pub trait HttpExecutor {
    fn execute(&mut self, request: RestRequest) -> Result<RestResponse, UcelError>;
}

#[derive(Debug, Clone, Deserialize)]
struct SbivcErrorBody {
    code: Option<String>,
    error_code: Option<String>,
    field: Option<String>,
    message: Option<String>,
}

pub struct SbivcRestClient<E> {
    endpoints: Vec<RestEndpoint>,
    executor: E,
}

impl<E: HttpExecutor> SbivcRestClient<E> {
    pub fn new(catalog_json: &str, executor: E) -> Result<Self, UcelError> {
        let catalog: Catalog = serde_json::from_str(catalog_json)
            .map_err(|e| UcelError::new(ErrorCode::CatalogInvalid, e.to_string()))?;
        Ok(Self {
            endpoints: catalog.rest_endpoints,
            executor,
        })
    }

    pub fn call<T: DeserializeOwned>(
        &mut self,
        id: &str,
        auth: Option<AuthContext>,
    ) -> Result<T, UcelError> {
        let endpoint = self
            .endpoints
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown id={id}")))?;

        let requires_auth = endpoint.visibility == "private";
        if requires_auth && auth.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "private endpoint requires auth",
            ));
        }

        let response = self.executor.execute(RestRequest {
            method: endpoint.method.clone(),
            url: format!("{}{}", endpoint.base_url, endpoint.path),
            auth,
        })?;

        if (200..300).contains(&response.status) {
            return serde_json::from_slice::<T>(&response.body)
                .map_err(|e| UcelError::new(ErrorCode::Internal, e.to_string()));
        }

        let mut err = map_error(response.status, &response.body);
        err.retry_after_ms = response.retry_after_ms;
        Err(err)
    }
}

fn map_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        return UcelError::new(ErrorCode::RateLimited, "rate limited");
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream 5xx");
    }

    let parsed = serde_json::from_slice::<SbivcErrorBody>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|value| value.error_code.as_deref().or(value.code.as_deref()));
    let field = parsed.as_ref().and_then(|value| value.field.as_deref());

    match (status, code, field) {
        (401, _, _) | (_, Some("AUTH_FAILED"), _) => {
            UcelError::new(ErrorCode::AuthFailed, "auth failed")
        }
        (403, _, _) | (_, Some("PERMISSION_DENIED"), _) => {
            UcelError::new(ErrorCode::PermissionDenied, "permission denied")
        }
        (_, Some("INVALID_ORDER"), _) | (_, _, Some("order")) => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        _ => UcelError::new(
            ErrorCode::Network,
            parsed
                .and_then(|value| value.message)
                .unwrap_or_else(|| "request failed".to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct SpyExecutor {
        calls: usize,
        next: Option<Result<RestResponse, UcelError>>,
    }

    impl HttpExecutor for SpyExecutor {
        fn execute(&mut self, _request: RestRequest) -> Result<RestResponse, UcelError> {
            self.calls += 1;
            self.next
                .take()
                .unwrap_or_else(|| Err(UcelError::new(ErrorCode::Internal, "missing response")))
        }
    }

    #[derive(Debug, Deserialize)]
    struct Pong {
        ok: bool,
    }

    fn fixture_catalog() -> &'static str {
        r#"{
          "rest_endpoints": [
            {
              "id": "crypto.public.rest.ping",
              "visibility": "public",
              "method": "GET",
              "base_url": "https://api.example.com",
              "path": "/v1/ping"
            },
            {
              "id": "crypto.private.rest.order.create",
              "visibility": "private",
              "method": "POST",
              "base_url": "https://api.example.com",
              "path": "/v1/order"
            }
          ]
        }"#
    }

    #[test]
    fn private_operation_is_rejected_before_transport_call() {
        let spy = SpyExecutor::default();
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.private.rest.order.create", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::MissingAuth);
        assert_eq!(client.executor.calls, 0);
    }

    #[test]
    fn public_operation_calls_transport_without_auth() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 200,
                body: br#"{"ok":true}"#.to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let response = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap();
        assert!(response.ok);
        assert_eq!(client.executor.calls, 1);
    }

    #[test]
    fn maps_rate_limit_and_retry_after() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 429,
                body: b"{}".to_vec(),
                retry_after_ms: Some(1500),
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::RateLimited);
        assert_eq!(error.retry_after_ms, Some(1500));
    }

    #[test]
    fn maps_5xx() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 503,
                body: b"{}".to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Upstream5xx);
    }

    #[test]
    fn maps_api_error_by_code_and_field() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Ok(RestResponse {
                status: 400,
                body: br#"{"error_code":"INVALID_ORDER","field":"order"}"#.to_vec(),
                retry_after_ms: None,
            })),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidOrder);
    }

    #[test]
    fn maps_timeout_without_string_matching() {
        let spy = SpyExecutor {
            calls: 0,
            next: Some(Err(UcelError::new(ErrorCode::Timeout, "timed out"))),
        };
        let mut client = SbivcRestClient::new(fixture_catalog(), spy).unwrap();

        let error = client
            .call::<Pong>("crypto.public.rest.ping", None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Timeout);
    }
}
