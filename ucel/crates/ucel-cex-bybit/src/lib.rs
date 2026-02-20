use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub requires_auth: bool,
}

const ENDPOINTS: [EndpointSpec; 77] = [
    EndpointSpec { id: "bybit.private.rest.account.fee-rate", method: "GET", path: "/v5/account/fee-rate", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.set-spot-hedge", method: "POST", path: "/v5/account/set-hedging-mode", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.transaction-log", method: "GET", path: "/v5/account/transaction-log", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.asset-info", method: "GET", path: "/v5/asset/transfer/query-asset-info", requires_auth: true },
    EndpointSpec { id: "bybit.public.rest.market.instrument", method: "GET", path: "/v5/market/instruments-info", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.kline", method: "GET", path: "/v5/market/kline", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.mark-kline", method: "GET", path: "/v5/market/mark-price-kline", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.orderbook", method: "GET", path: "/v5/market/orderbook", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.recent-trade", method: "GET", path: "/v5/market/recent-trade", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.tickers", method: "GET", path: "/v5/market/tickers", requires_auth: false },
    EndpointSpec { id: "bybit.private.rest.position.execution", method: "GET", path: "/v5/execution/list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.order-list", method: "GET", path: "/v5/order/history", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.open-order", method: "GET", path: "/v5/order/realtime", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.query-spot-quota", method: "GET", path: "/v5/order/spot-borrow-check", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.delivery", method: "GET", path: "/v5/asset/delivery-record", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.settlement", method: "GET", path: "/v5/asset/settlement-record", requires_auth: true },
    EndpointSpec { id: "bybit.public.rest.market.long-short-ratio", method: "GET", path: "/v5/market/account-ratio", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.delivery-price", method: "GET", path: "/v5/market/delivery-price", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.history-fund-rate", method: "GET", path: "/v5/market/funding/history", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.index-kline", method: "GET", path: "/v5/market/index-price-kline", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.open-interest", method: "GET", path: "/v5/market/open-interest", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.premium-index-kline", method: "GET", path: "/v5/market/premium-index-price-kline", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.risk-limit", method: "GET", path: "/v5/market/risk-limit", requires_auth: false },
    EndpointSpec { id: "bybit.private.rest.position.close-pnl", method: "GET", path: "/v5/position/closed-pnl", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.position-info", method: "GET", path: "/v5/position/list", requires_auth: true },
    EndpointSpec { id: "bybit.public.rest.market.iv", method: "GET", path: "/v5/market/historical-volatility", requires_auth: false },
    EndpointSpec { id: "bybit.private.rest.account.borrow-history", method: "GET", path: "/v5/account/borrow-history", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.coin-greeks", method: "GET", path: "/v5/account/coin-greeks", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.collateral-info", method: "GET", path: "/v5/account/collateral-info", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.account-info", method: "GET", path: "/v5/account/info", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.set-collateral", method: "POST", path: "/v5/account/set-collateral-switch", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.set-margin-mode", method: "POST", path: "/v5/account/set-margin-mode", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.upgrade-unified-account", method: "POST", path: "/v5/account/upgrade-to-uta", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.account.wallet", method: "GET", path: "/v5/account/wallet-balance", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.coin-info", method: "GET", path: "/v5/asset/coin/query-info", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.set-deposit-acct", method: "POST", path: "/v5/asset/deposit/deposit-to-account", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.master-deposit-addr", method: "GET", path: "/v5/asset/deposit/query-address", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.deposit-coin-spec", method: "GET", path: "/v5/asset/deposit/query-allowed-list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.internal-deposit-record", method: "GET", path: "/v5/asset/deposit/query-internal-record", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.deposit-record", method: "GET", path: "/v5/asset/deposit/query-record", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.sub-deposit-addr", method: "GET", path: "/v5/asset/deposit/query-sub-member-address", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.sub-deposit-record", method: "GET", path: "/v5/asset/deposit/query-sub-member-record", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.exchange", method: "GET", path: "/v5/asset/exchange/order-record", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.create-inter-transfer", method: "POST", path: "/v5/asset/transfer/inter-transfer", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.account-coin-balance", method: "GET", path: "/v5/asset/transfer/query-account-coin-balance", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.all-balance", method: "GET", path: "/v5/asset/transfer/query-account-coins-balance", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.inter-transfer-list", method: "GET", path: "/v5/asset/transfer/query-inter-transfer-list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.sub-uid-list", method: "GET", path: "/v5/asset/transfer/query-sub-member-list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.transferable-coin", method: "GET", path: "/v5/asset/transfer/query-transfer-coin-list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.unitransfer-list", method: "GET", path: "/v5/asset/transfer/query-universal-transfer-list", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.unitransfer", method: "POST", path: "/v5/asset/transfer/universal-transfer", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.asset.withdraw-record", method: "GET", path: "/v5/asset/withdraw/query-record", requires_auth: true },
    EndpointSpec { id: "bybit.public.rest.market.insurance", method: "GET", path: "/v5/market/insurance", requires_auth: false },
    EndpointSpec { id: "bybit.public.rest.market.time", method: "GET", path: "/v5/market/time", requires_auth: false },
    EndpointSpec { id: "bybit.private.rest.position.manual-add-margin", method: "POST", path: "/v5/position/add-margin", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.auto-add-margin", method: "POST", path: "/v5/position/set-auto-add-margin", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.leverage", method: "POST", path: "/v5/position/set-leverage", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.set-risk-limit", method: "POST", path: "/v5/position/set-risk-limit", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.tpsl-mode", method: "POST", path: "/v5/position/set-tpsl-mode", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.cross-isolate", method: "POST", path: "/v5/position/switch-isolated", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.position-mode", method: "POST", path: "/v5/position/switch-mode", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.position.trading-stop", method: "POST", path: "/v5/position/trading-stop", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.spot-margin-uta.vip-margin", method: "GET", path: "/v5/spot-margin-trade/data", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.spot-margin-uta.set-leverage", method: "POST", path: "/v5/spot-margin-trade/set-leverage", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.spot-margin-uta.status", method: "GET", path: "/v5/spot-margin-trade/state", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.spot-margin-uta.switch-mode", method: "POST", path: "/v5/spot-margin-trade/switch-mode", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.amend-order", method: "POST", path: "/v5/order/amend", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.batch-amend", method: "POST", path: "/v5/order/amend-batch", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.cancel-order", method: "POST", path: "/v5/order/cancel", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.cancel-all", method: "POST", path: "/v5/order/cancel-all", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.batch-cancel", method: "POST", path: "/v5/order/cancel-batch", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.create-order", method: "POST", path: "/v5/order/create", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.trade.batch-place", method: "POST", path: "/v5/order/create-batch", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.user.affiliate-info", method: "GET", path: "/v5/user/aff-customer-info", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.user.wallet-type", method: "GET", path: "/v5/user/get-member-type", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.user.apikey-info", method: "GET", path: "/v5/user/query-api", requires_auth: true },
    EndpointSpec { id: "bybit.private.rest.user.list-sub-apikeys", method: "GET", path: "/v5/user/sub-apikeys", requires_auth: true },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BybitRestResult {
    pub endpoint: String,
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BybitEnvelope<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i64,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    #[serde(default, rename = "result")]
    pub result: T,
    #[serde(default, rename = "time")]
    pub time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BybitRestResponse {
    Generic(BybitEnvelope<BybitRestResult>),
}

#[derive(Debug, Clone, Deserialize)]
struct BybitErrorBody {
    #[serde(rename = "retCode")]
    ret_code: Option<i64>,
    #[serde(rename = "retMsg")]
    ret_msg: Option<String>,
    #[serde(rename = "retryAfterMs")]
    retry_after_ms: Option<u64>,
}

#[derive(Clone)]
pub struct BybitRestAdapter {
    base_url: Arc<str>,
    pub http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl BybitRestAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Arc::from(base_url.into()),
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
    ) -> Result<BybitRestResponse, UcelError> {
        let spec = ENDPOINTS.iter().find(|it| it.id == endpoint_id).ok_or_else(|| {
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
            venue: "bybit".into(),
            policy_id: "default".into(),
            key_id,
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let response = transport
            .send_http(
                HttpRequest {
                    method: spec.method.into(),
                    path: format!("{}{}", self.base_url, spec.path),
                    body,
                },
                ctx,
            )
            .await?;

        if response.status >= 400 {
            return Err(map_bybit_http_error(response.status, &response.body));
        }

        let envelope: BybitEnvelope<BybitRestResult> = serde_json::from_slice(&response.body)
            .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))?;

        if envelope.ret_code != 0 {
            return Err(map_bybit_api_error(envelope.ret_code, &envelope.ret_msg));
        }

        Ok(BybitRestResponse::Generic(envelope))
    }
}

fn map_bybit_api_error(ret_code: i64, ret_msg: &str) -> UcelError {
    let code = match ret_code {
        10003 | 10004 | 10007 => ErrorCode::AuthFailed,
        10005 => ErrorCode::PermissionDenied,
        10006 => ErrorCode::RateLimited,
        10001 | 110001 | 110003 | 110004 | 110008 | 110017 | 110020 => ErrorCode::InvalidOrder,
        _ => ErrorCode::Internal,
    };
    UcelError::new(code, format!("bybit retCode={ret_code} {ret_msg}"))
}

pub fn map_bybit_http_error(status: u16, body: &[u8]) -> UcelError {
    let parsed = serde_json::from_slice::<BybitErrorBody>(body).ok();
    if status == 429 {
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.ban_risk = true;
        err.retry_after_ms = parsed.and_then(|p| p.retry_after_ms);
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    if let Some(body) = parsed {
        if let Some(ret_code) = body.ret_code {
            return map_bybit_api_error(ret_code, body.ret_msg.as_deref().unwrap_or(""));
        }
    }

    UcelError::new(ErrorCode::Internal, "unknown bybit error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use ucel_transport::{next_retry_delay_ms, HttpResponse, WsConnectRequest, WsStream};

    struct SpyTransport {
        calls: AtomicUsize,
        responses: Mutex<HashMap<String, Result<HttpResponse, UcelError>>>,
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
                Ok(HttpResponse {
                    status,
                    body: Bytes::copy_from_slice(body.as_bytes()),
                }),
            );
        }

        async fn set_error(&self, path: &str, err: UcelError) {
            self.responses.lock().await.insert(path.into(), Err(err));
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
            self.responses.lock().await.remove(&req.path).unwrap()
        }

        async fn connect_ws(
            &self,
            _req: WsConnectRequest,
            _ctx: RequestContext,
        ) -> Result<WsStream, UcelError> {
            Ok(WsStream { connected: true })
        }
    }

    fn fixture(id: &str) -> String {
        format!(
            "{{\"retCode\":0,\"retMsg\":\"OK\",\"result\":{{\"endpoint\":\"{id}\",\"ok\":true}},\"time\":1700000000}}"
        )
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rest_contract_all_endpoints_are_covered_by_fixture_driven_tests() {
        let transport = SpyTransport::new();
        let adapter = BybitRestAdapter::new("https://api.bybit.test");

        for spec in BybitRestAdapter::endpoint_specs() {
            let path = format!("https://api.bybit.test{}", spec.path);
            transport.set_response(&path, 200, &fixture(spec.id)).await;
            let key_id = spec.requires_auth.then(|| "k-1".to_string());
            let parsed = adapter
                .execute_rest(&transport, spec.id, None, key_id)
                .await
                .unwrap();
            match parsed {
                BybitRestResponse::Generic(envelope) => {
                    assert_eq!(envelope.result.endpoint, spec.id);
                    assert!(envelope.result.ok);
                }
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn private_endpoint_rejects_without_auth_and_transport_is_not_called() {
        let transport = SpyTransport::new();
        let adapter = BybitRestAdapter::new("https://api.bybit.test");

        let err = adapter
            .execute_rest(&transport, "bybit.private.rest.account.fee-rate", None, None)
            .await
            .unwrap_err();

        assert_eq!(err.code, ErrorCode::MissingAuth);
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_endpoint_works_without_key_path() {
        let transport = SpyTransport::new();
        let adapter = BybitRestAdapter::new("https://api.bybit.test");
        transport
            .set_response(
                "https://api.bybit.test/v5/market/time",
                200,
                &fixture("bybit.public.rest.market.time"),
            )
            .await;

        assert!(adapter
            .execute_rest(&transport, "bybit.public.rest.market.time", None, None)
            .await
            .is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maps_429_5xx_and_timeout() {
        let transport = SpyTransport::new();
        let adapter = BybitRestAdapter::new("https://api.bybit.test");

        transport
            .set_response(
                "https://api.bybit.test/v5/market/time",
                429,
                r#"{"retCode":10006,"retMsg":"too many visits","retryAfterMs":1200}"#,
            )
            .await;
        let rate = adapter
            .execute_rest(&transport, "bybit.public.rest.market.time", None, None)
            .await
            .unwrap_err();
        assert_eq!(rate.code, ErrorCode::RateLimited);
        assert_eq!(rate.retry_after_ms, Some(1200));

        transport
            .set_response(
                "https://api.bybit.test/v5/market/time",
                500,
                r#"{"retCode":0,"retMsg":"ok"}"#,
            )
            .await;
        let upstream = adapter
            .execute_rest(&transport, "bybit.public.rest.market.time", None, None)
            .await
            .unwrap_err();
        assert_eq!(upstream.code, ErrorCode::Upstream5xx);

        transport
            .set_error(
                "https://api.bybit.test/v5/market/time",
                UcelError::new(ErrorCode::Timeout, "timeout"),
            )
            .await;
        let timeout = adapter
            .execute_rest(&transport, "bybit.public.rest.market.time", None, None)
            .await
            .unwrap_err();
        assert_eq!(timeout.code, ErrorCode::Timeout);
    }

    #[test]
    fn maps_bybit_error_codes_without_message_matching() {
        assert_eq!(
            map_bybit_http_error(401, br#"{"retCode":10003,"retMsg":"api key invalid"}"#).code,
            ErrorCode::AuthFailed
        );
        assert_eq!(
            map_bybit_http_error(403, br#"{"retCode":10005,"retMsg":"permission denied"}"#).code,
            ErrorCode::PermissionDenied
        );
        assert_eq!(
            map_bybit_http_error(400, br#"{"retCode":110001,"retMsg":"order does not exist"}"#).code,
            ErrorCode::InvalidOrder
        );
    }

    #[test]
    fn retry_policy_respects_retry_after_for_429() {
        let policy = RetryPolicy {
            base_delay_ms: 100,
            max_delay_ms: 4_000,
            jitter_ms: 50,
            respect_retry_after: true,
        };
        assert_eq!(next_retry_delay_ms(&policy, 3, Some(777)), 777);
    }
}
