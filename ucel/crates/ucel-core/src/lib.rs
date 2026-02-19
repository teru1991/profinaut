use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub schema_version: String,
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
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeEvent {
    pub trade_id: String,
    pub price: f64,
    pub qty: f64,
    pub side: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub qty: f64,
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
    pub side: String,
    pub order_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderAck {
    pub order_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderState {
    pub order_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillEvent {
    pub order_id: String,
    pub fill_id: String,
    pub price: f64,
    pub qty: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

pub type Balances = Vec<Balance>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum ErrorCode {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OpName {
    FetchTicker,
    FetchTrades,
    FetchOrderbookSnapshot,
    SubscribeTicker,
    SubscribeTrades,
    SubscribeOrderbook,
    PlaceOrder,
    CancelOrder,
    FetchBalances,
    FetchOpenOrders,
    FetchFills,
}

impl fmt::Display for OpName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let canonical = match self {
            OpName::FetchTicker => "fetch_ticker",
            OpName::FetchTrades => "fetch_trades",
            OpName::FetchOrderbookSnapshot => "fetch_orderbook_snapshot",
            OpName::SubscribeTicker => "subscribe_ticker",
            OpName::SubscribeTrades => "subscribe_trades",
            OpName::SubscribeOrderbook => "subscribe_orderbook",
            OpName::PlaceOrder => "place_order",
            OpName::CancelOrder => "cancel_order",
            OpName::FetchBalances => "fetch_balances",
            OpName::FetchOpenOrders => "fetch_open_orders",
            OpName::FetchFills => "fetch_fills",
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
            format!("operation {} is not allowlisted", op),
        ))
    }
}

pub fn check_execution_mode(policy: &RuntimePolicy, op: OpName) -> Result<(), UcelError> {
    if policy.mode == ExecutionMode::Live || !default_requires_auth(op) {
        return Ok(());
    }
    Err(UcelError::new(
        ErrorCode::DryRunOnly,
        format!("operation {} blocked in dry-run mode", op),
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
        write!(f, "ResolvedSecret(api_key=***, api_secret=***, passphrase=***)")
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
}
