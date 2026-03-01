pub mod decimal;
pub mod order_gate;
pub mod symbol;
pub mod types;
pub mod value;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
pub use types::{Decimal, OrderStatus, OrderType, SchemaVersion, Side};
pub use value::{Notional, Price, Qty, StepSize, TickSize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub schema_version: SchemaVersion,
    pub meta: Meta,
    pub data: T,
    pub quality: Quality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub venue: String,
    pub symbol: String,
    pub venue_symbol: String,
    pub ts_recv: u64,
    pub ts_event: Option<u64>,
    pub trace_id: String,
    pub request_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Quality {
    pub is_stale: bool,
    pub delay_ms: u64,
    pub missing_fields: Vec<String>,
    pub anomaly_flags: Vec<String>,
    pub parse_failures_recent: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerSnapshot {
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeEvent {
    pub trade_id: String,
    pub price: Decimal,
    pub qty: Decimal,
    pub side: Side,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub qty: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookDelta {
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub sequence_start: u64,
    pub sequence_end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderIntent {
    pub client_order_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderAck {
    pub order_id: String,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderState {
    pub order_id: String,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillEvent {
    pub order_id: String,
    pub fill_id: String,
    pub price: Decimal,
    pub qty: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
}

pub type Balances = Vec<Balance>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum ErrorCode {
    #[error("CATALOG_INVALID")]
    CatalogInvalid,
    #[error("CATALOG_DUPLICATE_ID")]
    CatalogDuplicateId,
    #[error("CATALOG_MISSING_FIELD")]
    CatalogMissingField,
    #[error("NOT_SUPPORTED")]
    NotSupported,
    #[error("NOT_ALLOWED_OP")]
    NotAllowedOp,
    #[error("FEATURE_DISABLED")]
    FeatureDisabled,
    #[error("DRY_RUN_ONLY")]
    DryRunOnly,
    #[error("MISSING_AUTH")]
    MissingAuth,
    #[error("RATE_LIMITED")]
    RateLimited,
    #[error("AUTH_FAILED")]
    AuthFailed,
    #[error("PERMISSION_DENIED")]
    PermissionDenied,
    #[error("INVALID_ORDER")]
    InvalidOrder,
    #[error("TIMEOUT")]
    Timeout,
    #[error("NETWORK")]
    Network,
    #[error("UPSTREAM_5XX")]
    Upstream5xx,
    #[error("WS_PROTOCOL_VIOLATION")]
    WsProtocolViolation,
    #[error("DESYNC")]
    Desync,
    #[error("INTERNAL")]
    Internal,
    #[error("REGISTRY_INVALID_CATALOG")]
    RegistryInvalidCatalog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("{code}: {message}")]
pub struct UcelError {
    pub code: ErrorCode,
    pub message: String,
    pub retry_after_ms: Option<u64>,
    pub ban_risk: bool,
    pub key_specific: bool,
}

impl UcelError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retry_after_ms: None,
            ban_risk: false,
            key_specific: false,
        }
    }

    pub fn is_retryable(&self) -> bool {
        use ErrorCode::*;
        matches!(
            self.code,
            RateLimited | Timeout | Network | Upstream5xx | Desync
        )
    }

    pub fn with_retry_after_ms(mut self, milliseconds: u64) -> Self {
        self.retry_after_ms = Some(milliseconds);
        self
    }

    pub fn with_ban_risk(mut self, ban_risk: bool) -> Self {
        self.ban_risk = ban_risk;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OpName {
    FetchTicker,
    FetchStatus,
    FetchTrades,
    FetchKlines,
    FetchOrderbookSnapshot,
    SubscribeTicker,
    SubscribeTrades,
    SubscribeOrderbook,
    SubscribeExecutionEvents,
    SubscribeOrderEvents,
    SubscribePositionEvents,
    CreateWsAuthToken,
    ExtendWsAuthToken,
    PlaceOrder,
    AmendOrder,
    CancelOrder,
    FetchBalances,
    FetchMarginStatus,
    FetchOpenOrders,
    FetchLatestExecutions,
    FetchFills,
    FetchOpenPositions,
    FetchPositionSummary,
    ClosePositionByOrder,
}

impl fmt::Display for OpName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let canonical = match self {
            OpName::FetchTicker => "fetch_ticker",
            OpName::FetchStatus => "fetch_status",
            OpName::FetchTrades => "fetch_trades",
            OpName::FetchKlines => "fetch_klines",
            OpName::FetchOrderbookSnapshot => "fetch_orderbook_snapshot",
            OpName::SubscribeTicker => "subscribe_ticker",
            OpName::SubscribeTrades => "subscribe_trades",
            OpName::SubscribeOrderbook => "subscribe_orderbook",
            OpName::SubscribeExecutionEvents => "subscribe_execution_events",
            OpName::SubscribeOrderEvents => "subscribe_order_events",
            OpName::SubscribePositionEvents => "subscribe_position_events",
            OpName::CreateWsAuthToken => "create_ws_auth_token",
            OpName::ExtendWsAuthToken => "extend_ws_auth_token",
            OpName::PlaceOrder => "place_order",
            OpName::AmendOrder => "amend_order",
            OpName::CancelOrder => "cancel_order",
            OpName::FetchBalances => "fetch_balances",
            OpName::FetchMarginStatus => "fetch_margin_status",
            OpName::FetchOpenOrders => "fetch_open_orders",
            OpName::FetchLatestExecutions => "fetch_latest_executions",
            OpName::FetchFills => "fetch_fills",
            OpName::FetchOpenPositions => "fetch_open_positions",
            OpName::FetchPositionSummary => "fetch_position_summary",
            OpName::ClosePositionByOrder => "close_position_by_order",
        };
        write!(f, "{canonical}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    pub schema_version: String,
    pub kind: String,
    pub name: String,
    pub marketdata: MarketDataCapabilities,
    pub trading: Option<TradingCapabilities>,
    pub auth: Option<AuthCapabilities>,
    pub rate_limit: Option<RateLimitCapabilities>,
    pub operational: Option<OperationalCapabilities>,
    pub safe_defaults: SafeDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDataCapabilities {
    pub rest: bool,
    pub ws: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TradingCapabilities {
    pub place_order: bool,
    pub cancel_order: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthCapabilities {
    pub api_key: bool,
    pub passphrase: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RateLimitCapabilities {
    pub retry_after_header: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OperationalCapabilities {
    pub supports_health: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeDefaults {
    pub marketdata_default_on: bool,
    pub execution_default_dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailoverPolicy {
    pub cooldown_ms: u64,
    pub max_consecutive_failures: u32,
    pub respect_retry_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    DryRun,
    Live,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePolicy {
    pub policy_id: String,
    pub allowed_ops: Vec<OpName>,
    pub failover: FailoverPolicy,
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpMeta {
    pub op: OpName,
    pub requires_auth: bool,
}

pub fn default_requires_auth(op: OpName) -> bool {
    matches!(
        op,
        OpName::PlaceOrder
            | OpName::CancelOrder
            | OpName::FetchBalances
            | OpName::FetchOpenOrders
            | OpName::FetchFills
    )
}

pub fn is_op_allowed(policy: &RuntimePolicy, op: OpName) -> Result<(), UcelError> {
    if policy.allowed_ops.contains(&op) {
        Ok(())
    } else {
        Err(UcelError::new(
            ErrorCode::NotAllowedOp,
            format!("operation {op} is not allowlisted"),
        ))
    }
}

pub fn check_execution_mode(policy: &RuntimePolicy, op: OpName) -> Result<(), UcelError> {
    if policy.mode == ExecutionMode::Live || !default_requires_auth(op) {
        return Ok(());
    }
    Err(UcelError::new(
        ErrorCode::DryRunOnly,
        format!("operation {op} blocked in dry-run mode"),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyScope {
    ReadOnly,
    Trade,
    Withdraw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRef {
    pub key_id: String,
    pub secret_ref: String,
    pub scope: KeyScope,
    pub account_id: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSecret {
    pub api_key: String,
    pub api_secret: Option<String>,
    pub passphrase: Option<String>,
}

impl fmt::Debug for ResolvedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedSecret")
            .field("api_key", &"***")
            .field("api_secret", &self.api_secret.as_ref().map(|_| "***"))
            .field("passphrase", &self.passphrase.as_ref().map(|_| "***"))
            .finish()
    }
}

impl fmt::Display for ResolvedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ResolvedSecret(api_key=***, api_secret=***, passphrase=***)"
        )
    }
}

pub trait SecretRefResolver {
    fn resolve(&self, key_ref: &KeyRef) -> Result<ResolvedSecret, UcelError>;
}

pub trait Exchange {
    fn name(&self) -> &'static str;
    fn execute(&self, op: OpName) -> Result<(), UcelError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn op_names_display_are_canonical() {
        assert_eq!(OpName::FetchTicker.to_string(), "fetch_ticker");
        assert_eq!(OpName::PlaceOrder.to_string(), "place_order");
    }

    #[test]
    fn dry_run_blocks_private_ops() {
        let policy = RuntimePolicy {
            policy_id: "p1".into(),
            allowed_ops: vec![OpName::PlaceOrder],
            failover: FailoverPolicy {
                cooldown_ms: 10,
                max_consecutive_failures: 2,
                respect_retry_after: true,
            },
            mode: ExecutionMode::DryRun,
        };
        let err = check_execution_mode(&policy, OpName::PlaceOrder).unwrap_err();
        assert_eq!(err.code, ErrorCode::DryRunOnly);
    }

    #[test]
    fn secret_formatting_masks_values() {
        let secret = ResolvedSecret {
            api_key: "real_key".into(),
            api_secret: Some("real_secret".into()),
            passphrase: Some("real_pass".into()),
        };
        let dbg = format!("{secret:?}");
        let disp = format!("{secret}");
        assert!(!dbg.contains("real_secret"));
        assert!(!disp.contains("real_pass"));
        assert!(dbg.contains("***"));
    }

    #[test]
    fn enum_and_schema_contract_is_forward_compatible() {
        let side: Side = serde_json::from_value(json!("unexpected_side")).unwrap();
        assert_eq!(side, Side::Unknown);

        let status: OrderStatus = serde_json::from_value(json!("unexpected_status")).unwrap();
        assert_eq!(status, OrderStatus::Unknown);

        let schema_version: SchemaVersion = serde_json::from_value(json!("1.2.3")).unwrap();
        assert_eq!(schema_version, SchemaVersion::parse("1.2.3").unwrap());
    }

    #[test]
    fn ucel_error_retry_helpers_work() {
        let err = UcelError::new(ErrorCode::Timeout, "timeout")
            .with_retry_after_ms(1000)
            .with_ban_risk(true);

        assert!(err.is_retryable());
        assert_eq!(err.retry_after_ms, Some(1000));
        assert!(err.ban_risk);
    }
}
