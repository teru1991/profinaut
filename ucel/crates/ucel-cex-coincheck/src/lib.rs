use bytes::Bytes;
use serde::Deserialize;
use std::collections::HashMap;
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
pub struct CoincheckRestAdapter {
    pub base_url: Arc<str>,
    pub endpoints: Arc<Vec<EndpointSpec>>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl CoincheckRestAdapter {
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

    pub fn endpoint_specs(&self) -> &[EndpointSpec] {
        &self.endpoints
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        path_params: &HashMap<String, String>,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<CoincheckRestResponse, UcelError> {
        let spec = self
            .endpoints
            .iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown endpoint: {endpoint_id}")))?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: OpName::FetchStatus,
            venue: "coincheck".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let mut path = spec.path.clone();
        for (k, v) in path_params {
            path = path.replace(&format!("{{{k}}}"), v);
        }

        let response = transport
            .send_http(
                HttpRequest {
                    method: spec.method.clone(),
                    path: format!("{}{}", self.base_url, path),
                    body,
                },
                ctx,
            )
            .await?;

        if response.status >= 400 {
            return Err(map_coincheck_http_error(response.status, &response.body));
        }

        parse_response(endpoint_id, &response.body)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoincheckRestResponse {
    PublicTicker(SuccessOnly),
    PublicTrades(SuccessOnly),
    PublicOrderBooks(SuccessOnly),
    PublicOrderRate(SuccessOnly),
    PublicRatePair(SuccessOnly),
    PublicExchangeStatus(SuccessOnly),
    PrivateExchangeOrdersPost(SuccessOnly),
    PrivateExchangeOrderByIdGet(SuccessOnly),
    PrivateExchangeOrdersOpensGet(SuccessOnly),
    PrivateExchangeOrderByIdDelete(SuccessOnly),
    PrivateExchangeOrderCancelStatusGet(SuccessOnly),
    PrivateExchangeOrdersTransactionsGet(SuccessOnly),
    PrivateExchangeOrdersTransactionsPaginationGet(SuccessOnly),
    PrivateAccountsBalanceGet(SuccessOnly),
    PrivateSendMoneyPost(SuccessOnly),
    PrivateSendMoneyGet(SuccessOnly),
    PrivateDepositMoneyGet(SuccessOnly),
    PrivateAccountsGet(SuccessOnly),
    PrivateBankAccountsGet(SuccessOnly),
    PrivateBankAccountsPost(SuccessOnly),
    PrivateBankAccountByIdDelete(SuccessOnly),
    PrivateWithdrawsGet(SuccessOnly),
    PrivateWithdrawsPost(SuccessOnly),
    PrivateWithdrawByIdDelete(SuccessOnly),
    OtherAuthHeaders(SuccessOnly),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SuccessOnly {
    pub success: bool,
}

fn parse_response(endpoint_id: &str, bytes: &[u8]) -> Result<CoincheckRestResponse, UcelError> {
    let parse = |raw: &[u8]| {
        serde_json::from_slice::<SuccessOnly>(raw)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
    };

    Ok(match endpoint_id {
        "coincheck.rest.public.ticker.get" => CoincheckRestResponse::PublicTicker(parse(bytes)?),
        "coincheck.rest.public.trades.get" => CoincheckRestResponse::PublicTrades(parse(bytes)?),
        "coincheck.rest.public.order_books.get" => CoincheckRestResponse::PublicOrderBooks(parse(bytes)?),
        "coincheck.rest.public.exchange.orders.rate.get" => CoincheckRestResponse::PublicOrderRate(parse(bytes)?),
        "coincheck.rest.public.rate.pair.get" => CoincheckRestResponse::PublicRatePair(parse(bytes)?),
        "coincheck.rest.public.exchange_status.get" => CoincheckRestResponse::PublicExchangeStatus(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.post" => CoincheckRestResponse::PrivateExchangeOrdersPost(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.id.get" => CoincheckRestResponse::PrivateExchangeOrderByIdGet(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.opens.get" => CoincheckRestResponse::PrivateExchangeOrdersOpensGet(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.id.delete" => CoincheckRestResponse::PrivateExchangeOrderByIdDelete(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.cancel_status.get" => CoincheckRestResponse::PrivateExchangeOrderCancelStatusGet(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.transactions.get" => CoincheckRestResponse::PrivateExchangeOrdersTransactionsGet(parse(bytes)?),
        "coincheck.rest.private.exchange.orders.transactions_pagination.get" => CoincheckRestResponse::PrivateExchangeOrdersTransactionsPaginationGet(parse(bytes)?),
        "coincheck.rest.private.accounts.balance.get" => CoincheckRestResponse::PrivateAccountsBalanceGet(parse(bytes)?),
        "coincheck.rest.private.send_money.post" => CoincheckRestResponse::PrivateSendMoneyPost(parse(bytes)?),
        "coincheck.rest.private.send_money.get" => CoincheckRestResponse::PrivateSendMoneyGet(parse(bytes)?),
        "coincheck.rest.private.deposit_money.get" => CoincheckRestResponse::PrivateDepositMoneyGet(parse(bytes)?),
        "coincheck.rest.private.accounts.get" => CoincheckRestResponse::PrivateAccountsGet(parse(bytes)?),
        "coincheck.rest.private.bank_accounts.get" => CoincheckRestResponse::PrivateBankAccountsGet(parse(bytes)?),
        "coincheck.rest.private.bank_accounts.post" => CoincheckRestResponse::PrivateBankAccountsPost(parse(bytes)?),
        "coincheck.rest.private.bank_accounts.id.delete" => CoincheckRestResponse::PrivateBankAccountByIdDelete(parse(bytes)?),
        "coincheck.rest.private.withdraws.get" => CoincheckRestResponse::PrivateWithdrawsGet(parse(bytes)?),
        "coincheck.rest.private.withdraws.post" => CoincheckRestResponse::PrivateWithdrawsPost(parse(bytes)?),
        "coincheck.rest.private.withdraws.id.delete" => CoincheckRestResponse::PrivateWithdrawByIdDelete(parse(bytes)?),
        "coincheck.rest.other.auth.headers" => CoincheckRestResponse::OtherAuthHeaders(parse(bytes)?),
        _ => return Err(UcelError::new(ErrorCode::NotSupported, "unknown endpoint id")),
    })
}

#[derive(Debug, Deserialize)]
struct CoincheckErrorBody {
    code: Option<String>,
    error_code: Option<String>,
}

pub fn map_coincheck_http_error(status: u16, body: &[u8]) -> UcelError {
    if status == 429 {
        let retry_after_ms = std::str::from_utf8(body)
            .ok()
            .and_then(|s| s.strip_prefix("retry_after_ms="))
            .and_then(|s| s.parse::<u64>().ok());
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.retry_after_ms = retry_after_ms;
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let parsed = serde_json::from_slice::<CoincheckErrorBody>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|e| e.code.as_deref().or(e.error_code.as_deref()))
        .unwrap_or_default();

    let mapped = match (status, code) {
        (401, _) | (_, "authentication_error") | (_, "invalid_auth") => ErrorCode::AuthFailed,
        (403, _) | (_, "permission_denied") | (_, "forbidden") => ErrorCode::PermissionDenied,
        (400, _) | (404, _) | (_, "invalid_order") | (_, "invalid_request") => ErrorCode::InvalidOrder,
        _ => ErrorCode::Network,
    };
    UcelError::new(mapped, format!("coincheck http error status={status}"))
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
    auth: CatalogAuth,
}

#[derive(Debug, Deserialize)]
struct CatalogAuth {
    #[serde(rename = "type")]
    auth_type: String,
}

fn load_endpoint_specs() -> Vec<EndpointSpec> {
    let raw = include_str!("../../../../docs/exchanges/coincheck/catalog.json");
    let catalog: Catalog = serde_json::from_str(raw).expect("valid coincheck catalog");
    catalog
        .rest_endpoints
        .into_iter()
        .map(|entry| EndpointSpec {
            id: entry.id,
            method: entry.method,
            path: entry.path,
            requires_auth: entry.auth.auth_type == "signature",
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
        last_ctx: Mutex<Option<RequestContext>>,
        response: Mutex<Option<Result<HttpResponse, UcelError>>>,
    }

    impl SpyTransport {
        fn with_response(resp: Result<HttpResponse, UcelError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                last_ctx: Mutex::new(None),
                response: Mutex::new(Some(resp)),
            }
        }
    }

    impl Transport for SpyTransport {
        async fn send_http(&self, _req: HttpRequest, ctx: RequestContext) -> Result<HttpResponse, UcelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_ctx.lock().unwrap() = Some(ctx);
            self.response.lock().unwrap().take().unwrap()
        }

        async fn connect_ws(&self, _req: WsConnectRequest, _ctx: RequestContext) -> Result<WsStream, UcelError> {
            Ok(WsStream::default())
        }
    }

    #[test]
    fn loads_all_rest_rows_from_catalog() {
        let adapter = CoincheckRestAdapter::new("http://localhost");
        assert_eq!(adapter.endpoint_specs().len(), 25);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_5xx_timeout_and_private_preflight() {
        let e429 = map_coincheck_http_error(429, b"retry_after_ms=1200");
        assert_eq!(e429.code, ErrorCode::RateLimited);
        assert_eq!(e429.retry_after_ms, Some(1200));

        let e5xx = map_coincheck_http_error(503, br#"{"code":"x"}"#);
        assert_eq!(e5xx.code, ErrorCode::Upstream5xx);

        let adapter = CoincheckRestAdapter::new("http://localhost");
        let transport = SpyTransport::with_response(Err(UcelError::new(ErrorCode::Timeout, "timeout")));
        let err = adapter
            .execute_rest(
                &transport,
                "coincheck.rest.public.ticker.get",
                &HashMap::new(),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Timeout);

        let private_transport = SpyTransport::default();
        let err = adapter
            .execute_rest(
                &private_transport,
                "coincheck.rest.private.accounts.get",
                &HashMap::new(),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(private_transport.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_ops_never_require_key_path() {
        let adapter = CoincheckRestAdapter::new("http://localhost");
        let transport = SpyTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: Bytes::from_static(br#"{"success":true}"#),
        }));
        let _ = adapter
            .execute_rest(
                &transport,
                "coincheck.rest.public.ticker.get",
                &HashMap::new(),
                None,
                None,
            )
            .await
            .unwrap();

        let ctx = transport.last_ctx.lock().unwrap().clone().unwrap();
        assert!(!ctx.requires_auth);
        assert!(ctx.key_id.is_none());
    }
}
