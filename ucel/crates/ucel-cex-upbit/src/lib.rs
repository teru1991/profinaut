use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use ucel_core::{ErrorCode, OpName, UcelError};
use ucel_transport::{enforce_auth_boundary, HttpRequest, RequestContext, RetryPolicy, Transport};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogRestEndpoint>,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogRestEndpoint {
    id: String,
    method: String,
    base_url: String,
    path: String,
    visibility: String,
}

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub id: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
    pub requires_auth: bool,
    pub op: OpName,
}

pub fn endpoint_specs() -> Vec<EndpointSpec> {
    let catalog: Catalog = serde_json::from_str(include_str!("../../../../docs/exchanges/upbit/catalog.json"))
        .expect("upbit catalog must be valid json");

    catalog
        .rest_endpoints
        .into_iter()
        .map(|e| EndpointSpec {
            op: op_for_id(&e.id),
            requires_auth: e.visibility == "private",
            id: e.id,
            method: e.method,
            base_url: e.base_url,
            path: e.path,
        })
        .collect()
}

#[derive(Debug, Clone)]
pub enum UpbitRestResponse {
    Markets(Vec<Market>),
    Tickers(Vec<Ticker>),
    Trades(Vec<Trade>),
    Orderbook(Vec<OrderbookSnapshot>),
    Candles(Vec<Candle>),
    Accounts(Vec<Account>),
    CreateOrder(Order),
    CancelOrder(Order),
    OpenOrders(Vec<Order>),
    ClosedOrders(Vec<Order>),
    OrderChance(OrderChance),
    Withdraws(Vec<Withdraw>),
    WithdrawCoin(Withdraw),
    Deposits(Vec<Deposit>),
    DepositAddress(DepositAddress),
    TravelRuleVasps(Vec<TravelRuleVasp>),
    WalletStatus(Vec<WalletStatus>),
    ApiKeys(Vec<ApiKeyInfo>),
}

#[derive(Clone)]
pub struct UpbitRestAdapter {
    http_client: reqwest::Client,
    pub retry_policy: RetryPolicy,
}

impl UpbitRestAdapter {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");

        Self {
            http_client,
            retry_policy: RetryPolicy {
                base_delay_ms: 100,
                max_delay_ms: 5_000,
                jitter_ms: 20,
                respect_retry_after: true,
            },
        }
    }

    pub fn endpoint_specs(&self) -> Vec<EndpointSpec> {
        endpoint_specs()
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    pub async fn execute_rest<T: Transport>(
        &self,
        transport: &T,
        endpoint_id: &str,
        body: Option<Bytes>,
        key_id: Option<String>,
    ) -> Result<UpbitRestResponse, UcelError> {
        let spec = endpoint_specs()
            .into_iter()
            .find(|s| s.id == endpoint_id)
            .ok_or_else(|| UcelError::new(ErrorCode::NotSupported, format!("unknown endpoint: {endpoint_id}")))?;

        let ctx = RequestContext {
            trace_id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            op: spec.op,
            venue: "upbit".into(),
            policy_id: "default".into(),
            key_id: if spec.requires_auth { key_id } else { None },
            requires_auth: spec.requires_auth,
        };
        enforce_auth_boundary(&ctx)?;

        let req = HttpRequest {
            method: spec.method,
            path: format!("{}{}", spec.base_url, spec.path),
            body,
        };

        let response = transport.send_http(req, ctx).await?;
        if response.status >= 400 {
            return Err(map_upbit_http_error(response.status, &response.body));
        }

        match endpoint_id {
            "quotation.public.rest.markets.list" => Ok(UpbitRestResponse::Markets(parse_json(&response.body)?)),
            "quotation.public.rest.ticker.pairs" => Ok(UpbitRestResponse::Tickers(parse_json(&response.body)?)),
            "quotation.public.rest.trades.recent" => Ok(UpbitRestResponse::Trades(parse_json(&response.body)?)),
            "quotation.public.rest.orderbook.snapshot" => Ok(UpbitRestResponse::Orderbook(parse_json(&response.body)?)),
            "quotation.public.rest.candles.minutes"
            | "quotation.public.rest.candles.days"
            | "quotation.public.rest.candles.weeks"
            | "quotation.public.rest.candles.months"
            | "quotation.public.rest.candles.years" => Ok(UpbitRestResponse::Candles(parse_json(&response.body)?)),
            "exchange.private.rest.accounts.list" => Ok(UpbitRestResponse::Accounts(parse_json(&response.body)?)),
            "exchange.private.rest.orders.create" => Ok(UpbitRestResponse::CreateOrder(parse_json(&response.body)?)),
            "exchange.private.rest.orders.cancel" => Ok(UpbitRestResponse::CancelOrder(parse_json(&response.body)?)),
            "exchange.private.rest.orders.open" => Ok(UpbitRestResponse::OpenOrders(parse_json(&response.body)?)),
            "exchange.private.rest.orders.closed" => Ok(UpbitRestResponse::ClosedOrders(parse_json(&response.body)?)),
            "exchange.private.rest.orders.chance" => Ok(UpbitRestResponse::OrderChance(parse_json(&response.body)?)),
            "exchange.private.rest.withdraws.list" => Ok(UpbitRestResponse::Withdraws(parse_json(&response.body)?)),
            "exchange.private.rest.withdraws.coin" => Ok(UpbitRestResponse::WithdrawCoin(parse_json(&response.body)?)),
            "exchange.private.rest.deposits.list" => Ok(UpbitRestResponse::Deposits(parse_json(&response.body)?)),
            "exchange.private.rest.deposits.address" => Ok(UpbitRestResponse::DepositAddress(parse_json(&response.body)?)),
            "exchange.private.rest.travelrule.vasps" => Ok(UpbitRestResponse::TravelRuleVasps(parse_json(&response.body)?)),
            "exchange.private.rest.service.walletstatus" => Ok(UpbitRestResponse::WalletStatus(parse_json(&response.body)?)),
            "exchange.private.rest.keys.list" => Ok(UpbitRestResponse::ApiKeys(parse_json(&response.body)?)),
            _ => Err(UcelError::new(ErrorCode::NotSupported, format!("unsupported endpoint: {endpoint_id}"))),
        }
    }
}

impl Default for UpbitRestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, UcelError> {
    serde_json::from_slice(bytes)
        .map_err(|e| UcelError::new(ErrorCode::Internal, format!("json parse error: {e}")))
}

#[derive(Debug, Deserialize)]
struct UpbitErrorEnvelope {
    error: Option<UpbitErrorBody>,
    name: Option<String>,
    code: Option<String>,
    retry_after_ms: Option<u64>,
    retry_after: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpbitErrorBody {
    name: Option<String>,
    code: Option<String>,
}

pub fn map_upbit_http_error(status: u16, body: &[u8]) -> UcelError {
    let payload = serde_json::from_slice::<UpbitErrorEnvelope>(body).ok();
    let code = payload
        .as_ref()
        .and_then(|p| p.error.as_ref().and_then(|e| e.name.clone()).or_else(|| p.error.as_ref().and_then(|e| e.code.clone())).or_else(|| p.name.clone()).or_else(|| p.code.clone()))
        .unwrap_or_default();

    if status == 429 {
        let mut err = UcelError::new(ErrorCode::RateLimited, "rate limited");
        err.ban_risk = true;
        err.retry_after_ms = payload
            .as_ref()
            .and_then(|p| p.retry_after_ms.or(p.retry_after));
        return err;
    }
    if status >= 500 {
        return UcelError::new(ErrorCode::Upstream5xx, "upstream server error");
    }

    let mut err = match code.as_str() {
        "invalid_access_key" | "jwt_verification" | "expired_access_key" => {
            UcelError::new(ErrorCode::AuthFailed, "authentication failed")
        }
        "out_of_scope" => UcelError::new(ErrorCode::PermissionDenied, "permission denied"),
        "insufficient_funds_bid" | "insufficient_funds_ask" | "under_min_total_bid" | "under_min_total_ask" | "validation_error" | "create_ask_error" | "create_bid_error" => {
            UcelError::new(ErrorCode::InvalidOrder, "invalid order")
        }
        "too_many_requests" => UcelError::new(ErrorCode::RateLimited, "rate limited"),
        _ => UcelError::new(
            ErrorCode::Internal,
            format!("upbit http error status={status} code={code}"),
        ),
    };

    err.key_specific = matches!(err.code, ErrorCode::AuthFailed | ErrorCode::PermissionDenied);
    err
}

fn op_for_id(id: &str) -> OpName {
    match id {
        "quotation.public.rest.ticker.pairs" => OpName::FetchTicker,
        "quotation.public.rest.trades.recent" => OpName::FetchTrades,
        "quotation.public.rest.orderbook.snapshot" => OpName::FetchOrderbookSnapshot,
        "quotation.public.rest.candles.minutes"
        | "quotation.public.rest.candles.days"
        | "quotation.public.rest.candles.weeks"
        | "quotation.public.rest.candles.months"
        | "quotation.public.rest.candles.years" => OpName::FetchKlines,
        "exchange.private.rest.accounts.list" => OpName::FetchBalances,
        "exchange.private.rest.orders.create" => OpName::PlaceOrder,
        "exchange.private.rest.orders.cancel" => OpName::CancelOrder,
        "exchange.private.rest.orders.open" => OpName::FetchOpenOrders,
        "exchange.private.rest.orders.closed" => OpName::FetchLatestExecutions,
        _ => OpName::FetchStatus,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    pub market: String,
    pub korean_name: String,
    pub english_name: String,
    #[serde(default)]
    pub market_warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    pub market: String,
    pub trade_price: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub trade_price: f64,
    pub trade_volume: f64,
    #[serde(default)]
    pub trade_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshot {
    pub market: String,
    pub orderbook_units: Vec<OrderbookUnit>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookUnit {
    pub ask_price: f64,
    pub bid_price: f64,
    pub ask_size: f64,
    pub bid_size: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Candle {
    pub candle_date_time_utc: String,
    pub opening_price: f64,
    pub trade_price: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub currency: String,
    pub balance: String,
    pub locked: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub uuid: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderChance {
    pub bid_fee: String,
    pub ask_fee: String,
    pub market: OrderChanceMarket,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderChanceMarket {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Withdraw {
    pub currency: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deposit {
    pub currency: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepositAddress {
    pub currency: String,
    pub deposit_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TravelRuleVasp {
    pub vasp_name: String,
    pub vasp_code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletStatus {
    pub currency: String,
    pub wallet_state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyInfo {
    pub access_key: String,
    pub expire_at: String,
}
