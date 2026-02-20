use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use ucel_core::{
    AuthCapabilities, Capabilities, ErrorCode, FailoverPolicy, MarketDataCapabilities, OpMeta,
    OpName, OperationalCapabilities, RateLimitCapabilities, RuntimePolicy, SafeDefaults,
    TradingCapabilities, UcelError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub id: String,
    pub venue: String,
    pub enabled: bool,
    pub policy: RuntimePolicy,
    pub auth: AuthConfigRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfigRef {
    pub key_pool: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExchangeCatalog {
    pub exchange: String,
    pub rest_endpoints: Vec<CatalogEntry>,
    pub ws_channels: Vec<CatalogEntry>,
    #[serde(default)]
    pub data_feeds: Vec<DataFeedEntry>,
}

pub type GmoCatalog = ExchangeCatalog;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DataFeedEntry {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub visibility: String,
    pub operation: Option<String>,
    pub method: Option<String>,
    pub base_url: Option<String>,
    pub path: Option<String>,
    pub ws_url: Option<String>,
    pub ws: Option<CatalogWs>,
    pub auth: CatalogAuth,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CatalogWs {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CatalogAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
}

pub fn load_catalog_from_path(path: &Path) -> Result<ExchangeCatalog, UcelError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to read {}: {e}", path.display()),
        )
    })?;
    let catalog: ExchangeCatalog = serde_json::from_str(&raw).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to parse {}: {e}", path.display()),
        )
    })?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

pub fn load_catalog_from_repo_root(
    repo_root: &Path,
    exchange: &str,
) -> Result<ExchangeCatalog, UcelError> {
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(exchange)
        .join("catalog.json");
    load_catalog_from_path(&path)
}

pub fn validate_catalog(catalog: &ExchangeCatalog) -> Result<(), UcelError> {
    if catalog.exchange.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "catalog.exchange must not be empty",
        ));
    }

    let mut seen = HashSet::new();
    for entry in catalog
        .rest_endpoints
        .iter()
        .chain(catalog.ws_channels.iter())
    {
        validate_entry(entry)?;
        if !seen.insert(entry.id.clone()) {
            return Err(UcelError::new(
                ErrorCode::CatalogDuplicateId,
                format!("duplicate id found: {}", entry.id),
            ));
        }
    }
    Ok(())
}

fn validate_entry(entry: &CatalogEntry) -> Result<(), UcelError> {
    if entry.id.trim().is_empty() || entry.visibility.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing required fields for id={}", entry.id),
        ));
    }

    let visibility = entry.visibility.to_ascii_lowercase();
    match visibility.as_str() {
        "public" => {
            if entry.auth.auth_type != "none" {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!(
                        "public endpoint must use auth.type=none for id={}",
                        entry.id
                    ),
                ));
            }
        }
        "private" => {
            if entry.auth.auth_type == "none" {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!(
                        "private endpoint must not use auth.type=none for id={}",
                        entry.id
                    ),
                ));
            }
        }
        _ => {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "invalid visibility={} for id={}",
                    entry.visibility, entry.id
                ),
            ));
        }
    }

    let resolved_ws_url = entry
        .ws_url
        .as_deref()
        .or(entry.ws.as_ref().map(|ws| ws.url.as_str()));
    match (&entry.method, &entry.base_url, &entry.path, resolved_ws_url) {
        (Some(method), Some(base_url), Some(path), None) => {
            if method.trim().is_empty() || base_url.trim().is_empty() || path.trim().is_empty() {
                return Err(UcelError::new(
                    ErrorCode::CatalogMissingField,
                    format!(
                        "rest endpoint has empty method/base_url/path for id={}",
                        entry.id
                    ),
                ));
            }
            if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid base_url for id={}: {base_url}", entry.id),
                ));
            }
            if !path.starts_with('/') {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid path for id={}: {path}", entry.id),
                ));
            }
        }
        (None, None, None, Some(ws_url)) => {
            if ws_url.trim().is_empty() {
                return Err(UcelError::new(
                    ErrorCode::CatalogMissingField,
                    format!("ws endpoint has empty ws_url for id={}", entry.id),
                ));
            }
            if !(ws_url.starts_with("wss://") || ws_url.starts_with("ws://")) {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid ws_url for id={}: {ws_url}", entry.id),
                ));
            }
        }
        _ => {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "catalog row must be either REST(method/base_url/path) or WS(ws.url/ws_url), id={}",
                    entry.id
                ),
            ));
        }
    }

    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_operation(entry)?;
    let requires_auth = entry.visibility.eq_ignore_ascii_case("private");
    Ok(OpMeta { op, requires_auth })
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }
    if let Some(op) = map_bitmex_operation_by_id(&entry.id) {
        return Ok(op);
    }
    map_operation_by_id(&entry.id)
}

fn map_bitmex_operation_by_id(id: &str) -> Option<OpName> {
    if let Some(channel) = id
        .strip_prefix("public.ws.")
        .and_then(|raw| raw.strip_suffix(".subscribe"))
    {
        return Some(map_bitmex_public_ws_channel(channel));
    }

    if let Some(channel) = id
        .strip_prefix("private.ws.")
        .and_then(|raw| raw.strip_suffix(".subscribe"))
    {
        return Some(map_bitmex_private_ws_channel(channel));
    }

    if !id.contains(".rest.") {
        return None;
    }

    let mut parts = id.split('.');
    let _visibility = parts.next()?;
    let transport = parts.next()?;
    if transport != "rest" {
        return None;
    }
    let resource = parts.next()?;
    let action = parts.next().unwrap_or_default();
    Some(map_bitmex_rest_resource(resource, action))
}

fn map_bitmex_public_ws_channel(channel: &str) -> OpName {
    if channel.starts_with("trade") {
        OpName::SubscribeTrades
    } else if channel.starts_with("orderbook") {
        OpName::SubscribeOrderbook
    } else if channel == "quote" || channel.starts_with("quotebin") || channel == "instrument" {
        OpName::SubscribeTicker
    } else {
        OpName::FetchStatus
    }
}

fn map_bitmex_private_ws_channel(channel: &str) -> OpName {
    match channel {
        "execution" | "transact" => OpName::SubscribeExecutionEvents,
        "order" => OpName::SubscribeOrderEvents,
        "position" | "margin" | "wallet" => OpName::SubscribePositionEvents,
        _ => OpName::FetchStatus,
    }
}

fn map_bitmex_rest_resource(resource: &str, action: &str) -> OpName {
    match resource {
        "order" => {
            if action == "order-new" {
                OpName::PlaceOrder
            } else if action == "order-amend" {
                OpName::AmendOrder
            } else if action == "order-cancel"
                || action == "order-cancelall"
                || action == "order-cancelallafter"
            {
                OpName::CancelOrder
            } else if action == "order-closeposition" {
                OpName::ClosePositionByOrder
            } else {
                OpName::FetchOpenOrders
            }
        }
        "position" => {
            if action == "position-get" {
                OpName::FetchOpenPositions
            } else {
                OpName::FetchPositionSummary
            }
        }
        "execution" => OpName::FetchFills,
        "trade" => OpName::FetchTrades,
        "quote" => OpName::FetchTicker,
        "orderbook" => OpName::FetchOrderbookSnapshot,
        "user" => {
            if action == "user-getmargin" {
                OpName::FetchMarginStatus
            } else if action.starts_with("user-getwallet")
                || action.starts_with("user-getdeposit")
                || action.contains("withdrawal")
                || action.contains("staking")
            {
                OpName::FetchBalances
            } else {
                OpName::FetchStatus
            }
        }
        "wallet" | "address" | "porl" => OpName::FetchBalances,
        "instrument" | "schema" | "stats" | "announcement" | "globalnotification"
        | "leaderboard" | "liquidation" | "insurance" | "funding" | "guild" | "chat" | "apikey"
        | "settlement" | "useraffiliates" | "userevent" => OpName::FetchStatus,
        _ => OpName::FetchStatus,
    }
}

fn map_operation_literal(operation: &str) -> Option<OpName> {
    match operation {
        "Get service status" | "Get FX API status" | "List futures instruments" => {
            Some(OpName::FetchStatus)
        }
        "Get ticker" | "Get FX ticker" | "Get ticker information" | "Get futures tickers" => {
            Some(OpName::FetchTicker)
        }
        "Get order book" | "Get FX order book" => Some(OpName::FetchOrderbookSnapshot),
        "Get recent trades" | "Get FX trades" => Some(OpName::FetchTrades),
        "Get candlesticks" | "Get FX klines" => Some(OpName::FetchKlines),
        "Create WS auth token" | "Create FX WS auth token" | "Get WS token" => {
            Some(OpName::CreateWsAuthToken)
        }
        "Extend WS auth token" => Some(OpName::ExtendWsAuthToken),
        "Get account assets"
        | "Get FX account assets"
        | "Get account balances"
        | "Get account information" => Some(OpName::FetchBalances),
        "Get margin status" => Some(OpName::FetchMarginStatus),
        "Get active orders" | "Get FX active orders" => Some(OpName::FetchOpenOrders),
        "Get execution history" => Some(OpName::FetchFills),
        "Get latest execution per order" => Some(OpName::FetchLatestExecutions),
        "Create order" | "Create FX order" | "Add order" | "Send futures order" => {
            Some(OpName::PlaceOrder)
        }
        "Amend order" => Some(OpName::AmendOrder),
        "Cancel order" | "Cancel FX order" => Some(OpName::CancelOrder),
        "Get open positions" | "Get FX open positions" => Some(OpName::FetchOpenPositions),
        "Get position summary" => Some(OpName::FetchPositionSummary),
        "Close position by order" | "Close FX position" => Some(OpName::ClosePositionByOrder),
        _ => None,
    }
}

fn map_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    let op = match id {
        "crypto.public.ws.ticker.update"
        | "fx.public.ws.ticker.update"
        | "futures.public.ws.other.market.ticker.subscribe" => OpName::SubscribeTicker,
        "crypto.public.ws.trades.update"
        | "fx.public.ws.trades.update"
        | "spot.public.ws.v1.market.trade.subscribe" => OpName::SubscribeTrades,
        "crypto.public.ws.orderbooks.update"
        | "fx.public.ws.orderbooks.update"
        | "spot.public.ws.v1.market.book.subscribe"
        | "spot.public.ws.v2.market.book.subscribe"
        | "futures.public.ws.other.market.book.subscribe" => OpName::SubscribeOrderbook,
        "crypto.private.ws.executionevents.update" | "fx.private.ws.executionevents.update" => {
            OpName::SubscribeExecutionEvents
        }
        "crypto.private.ws.orderevents.update"
        | "fx.private.ws.orderevents.update"
        | "spot.private.ws.v1.account.open_orders.subscribe" => OpName::SubscribeOrderEvents,
        "crypto.private.ws.positionevents.update"
        | "fx.private.ws.positionevents.update"
        | "futures.private.ws.other.account.open_positions.subscribe" => {
            OpName::SubscribePositionEvents
        }
        "spot.public.rest.assets.list"
        | "spot.public.rest.asset-pairs.list"
        | "spot.public.ws.v2.market.instrument.subscribe" => OpName::FetchStatus,
        "spot.private.rest.order.add"
        | "futures.private.rest.order.send"
        | "spot.private.ws.v1.trade.add_order.request"
        | "spot.private.ws.v2.trade.add_order" => OpName::PlaceOrder,
        _ => {
            return Err(UcelError::new(
                ErrorCode::NotSupported,
                format!("unsupported operation mapping for id={id}"),
            ));
        }
    };
    Ok(op)
}

pub fn capabilities_from_catalog(name: &str, catalog: &ExchangeCatalog) -> Capabilities {
    Capabilities {
        schema_version: "1.0.0".into(),
        kind: "exchange".into(),
        name: name.into(),
        marketdata: MarketDataCapabilities {
            rest: !catalog.rest_endpoints.is_empty(),
            ws: !catalog.ws_channels.is_empty(),
        },
        trading: Some(TradingCapabilities::default()),
        auth: Some(AuthCapabilities::default()),
        rate_limit: Some(RateLimitCapabilities::default()),
        operational: Some(OperationalCapabilities::default()),
        safe_defaults: SafeDefaults {
            marketdata_default_on: true,
            execution_default_dry_run: true,
        },
    }
}

pub fn default_policy(policy_id: &str) -> RuntimePolicy {
    RuntimePolicy {
        policy_id: policy_id.into(),
        allowed_ops: vec![
            OpName::FetchTicker,
            OpName::FetchTrades,
            OpName::FetchOrderbookSnapshot,
        ],
        failover: FailoverPolicy {
            cooldown_ms: 1_000,
            max_consecutive_failures: 3,
            respect_retry_after: true,
        },
        mode: ucel_core::ExecutionMode::DryRun,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let catalog = ExchangeCatalog {
            exchange: "gmo".into(),
            rest_endpoints: vec![CatalogEntry {
                id: "same".into(),
                visibility: "public".into(),
                operation: Some("Get ticker".into()),
                method: Some("GET".into()),
                base_url: Some("https://x".into()),
                path: Some("/ok".into()),
                ws_url: None,
                ws: None,
                auth: CatalogAuth {
                    auth_type: "none".into(),
                },
            }],
            ws_channels: vec![CatalogEntry {
                id: "same".into(),
                visibility: "public".into(),
                operation: None,
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://x".into()),
                ws: None,
                auth: CatalogAuth {
                    auth_type: "none".into(),
                },
            }],
            data_feeds: vec![],
        };
        let err = validate_catalog(&catalog).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogDuplicateId);
    }

    #[test]
    fn requires_auth_comes_from_visibility() {
        let private_entry = CatalogEntry {
            id: "crypto.private.ws.executionevents.update".into(),
            visibility: "private".into(),
            operation: None,
            method: None,
            base_url: None,
            path: None,
            ws_url: Some("wss://api.coin.z.com/ws/private/v1/xxx".into()),
            ws: None,
            auth: CatalogAuth {
                auth_type: "token".into(),
            },
        };
        let public_entry = CatalogEntry {
            id: "crypto.public.rest.ticker.get".into(),
            visibility: "public".into(),
            operation: Some("Get ticker".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.coin.z.com".into()),
            path: Some("/public/v1/ticker".into()),
            ws_url: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };

        assert!(op_meta_from_entry(&private_entry).unwrap().requires_auth);
        assert!(!op_meta_from_entry(&public_entry).unwrap().requires_auth);
    }

    #[test]
    fn loads_gmocoin_catalog() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "gmocoin").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 30);
        assert_eq!(catalog.ws_channels.len(), 12);
    }

    #[test]
    fn loads_kraken_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "kraken").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 10);
        assert_eq!(catalog.ws_channels.len(), 10);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            assert!(
                map_operation(entry).is_ok(),
                "missing op mapping for {}",
                entry.id
            );
        }
    }

    #[test]
    fn loads_bitmex_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "bitmex").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 95);
        assert_eq!(catalog.ws_channels.len(), 30);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            assert!(
                map_operation(entry).is_ok(),
                "missing op mapping for {}",
                entry.id
            );
        }
    }
}
