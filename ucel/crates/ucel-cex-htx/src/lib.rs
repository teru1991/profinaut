use bytes::Bytes;
use serde::de::DeserializeOwned;
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HtxRestResponse {
    pub status: Option<String>,
    pub ch: Option<String>,
    pub ts: Option<u64>,
    pub data: Option<Vec<HtxDataItem>>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, HtxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HtxDataItem {
    #[serde(flatten)]
    pub fields: BTreeMap<String, HtxField>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum HtxField {
    String(String),
    Number(f64),
    Bool(bool),
    Object(BTreeMap<String, HtxField>),
    Array(Vec<HtxField>),
    Null(()),
}

#[derive(Clone)]
pub struct HtxRestAdapter {
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl HtxRestAdapter {
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
    ) -> Result<HtxRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|e| e.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown endpoint: {endpoint_id}")))?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "htx".into(),
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

        let resp = transport.send_http(req, ctx).await?;
        if resp.status >= 400 {
            return Err(map_htx_http_error(resp.status, &resp.body));
        }

        parse_json(&resp.body)
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct HtxErrorEnvelope {
    #[serde(rename = "err-code")]
    err_code: Option<String>,
    #[serde(rename = "error-code")]
    error_code: Option<String>,
    code: Option<String>,
    #[serde(rename = "status")]
    status_text: Option<String>,
}

pub fn map_htx_http_error(status: u16, body: &[u8]) -> UcelError {
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

    let envelope = serde_json::from_slice::<HtxErrorEnvelope>(body).ok();
    let code = envelope
        .as_ref()
        .and_then(|e| e.err_code.as_deref())
        .or_else(|| envelope.as_ref().and_then(|e| e.error_code.as_deref()))
        .or_else(|| envelope.as_ref().and_then(|e| e.code.as_deref()))
        .or_else(|| envelope.as_ref().and_then(|e| e.status_text.as_deref()))
        .unwrap_or_default()
        .to_ascii_uppercase();

    if status == 401 || status == 407 || code.contains("AUTH") || code.contains("SIGN") {
        return UcelError::new(ErrorCode::AuthFailed, format!("htx status={status} code={code}"));
    }
    if status == 403 || code.contains("PERMISSION") || code.contains("FORBIDDEN") {
        return UcelError::new(
            ErrorCode::PermissionDenied,
            format!("htx status={status} code={code}"),
        );
    }
    if status == 400 || status == 404 || status == 409 || status == 422 {
        return UcelError::new(ErrorCode::InvalidOrder, format!("htx status={status} code={code}"));
    }

    UcelError::new(ErrorCode::Network, format!("htx status={status} code={code}"))
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
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/htx/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid htx catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            base_url: entry.base_url,
            path: entry.path,
            requires_auth: entry.visibility.eq_ignore_ascii_case("private"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_catalog_rows() {
        let adapter = HtxRestAdapter::new();
        assert_eq!(adapter.endpoints.len(), 13);
    }
}
