use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use ucel_core::{ErrorCode, OpMeta, OpName, UcelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub id: String,
    pub venue: String,
    pub enabled: bool,
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
    #[serde(default)]
    pub visibility: Option<String>,
    pub visibility: String,
    #[serde(default)]
    pub operation: Option<String>,
    pub method: Option<String>,
    pub base_url: Option<String>,
    pub path: Option<String>,
    pub ws_url: Option<String>,
    pub ws: Option<CatalogWs>,
    pub auth: CatalogAuth,
    pub requires_auth: Option<bool>,
    pub auth: CatalogAuth,
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
            format!("read {}: {e}", path.display()),
        )
    })?;
    let catalog: ExchangeCatalog = serde_json::from_str(&raw).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("parse {}: {e}", path.display()),
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
        .join(exchange.to_ascii_lowercase())
        .join("catalog.json");
    load_catalog_from_path(&path)
    load_catalog_from_path(
        &repo_root
            .join("docs")
            .join("exchanges")
            .join(exchange.to_ascii_lowercase())
            .join("catalog.json"),
    )
}

pub fn validate_catalog(catalog: &ExchangeCatalog) -> Result<(), UcelError> {
    if catalog.exchange.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "catalog.exchange empty",
        ));
    }
    let mut seen = HashSet::new();
    for e in catalog
        .rest_endpoints
        .iter()
        .chain(catalog.ws_channels.iter())
    {
        if e.id.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                "entry.id empty",
            ));
        }
        if !seen.insert(e.id.clone()) {
    }
    Ok(())
}

fn validate_entry(entry: &CatalogEntry) -> Result<(), UcelError> {
    if entry.id.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "catalog entry id must not be empty",
        ));
    }

    let visibility = normalized_visibility(entry)?;
    match visibility.as_str() {
        "public" | "private" => {}
        _ => {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "invalid visibility={} for id={}",
                    entry.visibility.as_str(),
                    entry.id
                ),
            ));
        }
    }

    if let Some(requires_auth) = entry.requires_auth {
        let expected_requires_auth = visibility == "private";
        if requires_auth != expected_requires_auth {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth conflicts with visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, requires_auth
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
            if !is_placeholder(base_url)
                && !(base_url.starts_with("https://") || base_url.starts_with("http://"))
            {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid base_url for id={}: {base_url}", entry.id),
                ));
            }
            if !is_placeholder(path) && !path.starts_with('/') {
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
            if !is_placeholder(ws_url)
                && !(ws_url.starts_with("wss://")
                    || ws_url.starts_with("ws://")
                    || ws_url.starts_with("https://")
                    || ws_url.starts_with("http://"))
            {
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

fn is_placeholder(v: &str) -> bool {
    let t = v.trim();
    (t.starts_with("{{") && t.ends_with("}}")) || (t.starts_with("${") && t.ends_with("}"))
}

fn auth_type_requires_credentials(auth_type: &str) -> bool {
    matches!(auth_type, "apiKey" | "token" | "oauth2")
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_operation(entry)?;
    let requires_auth = normalized_visibility(entry)? == "private";
    Ok(OpMeta { op, requires_auth })
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if entry.id.starts_with("bybit.") {
        return map_bybit_operation(entry);
    }

    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }
    map_operation_by_id(&entry.id)
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
        | "Get account information"
        | "Account information" => Some(OpName::FetchBalances),
        "Get margin status" => Some(OpName::FetchMarginStatus),
        "Get active orders" | "Get FX active orders" => Some(OpName::FetchOpenOrders),
        "Get execution history" => Some(OpName::FetchFills),
        "Get latest execution per order" => Some(OpName::FetchLatestExecutions),
        "Create order" | "Create FX order" | "Add order" | "Send futures order"
        | "Place new order" => Some(OpName::PlaceOrder),
        "Amend order" => Some(OpName::AmendOrder),
        "Cancel order" | "Cancel FX order" => Some(OpName::CancelOrder),
        "Get open positions" | "Get FX open positions" => Some(OpName::FetchOpenPositions),
        "Get position summary" => Some(OpName::FetchPositionSummary),
        "Close position by order" | "Close FX position" => Some(OpName::ClosePositionByOrder),
        _ => None,
    }
}

fn is_placeholder(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed == "n/a" || trimmed == "N/A" || trimmed.contains("<") || trimmed.contains("{")
}

fn map_coinbase_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    let op = if id.contains(".ws.") {
        if id.contains("ticker") {
            OpName::SubscribeTicker
        } else if id.contains("candles") || id.contains("trades") {
            OpName::SubscribeTrades
        } else if id.contains("level2") || id.contains("book") {
            OpName::SubscribeOrderbook
        } else if id.contains("user") || id.contains("fills") {
            OpName::SubscribeExecutionEvents
        } else {
            OpName::FetchStatus
        }
    } else if id.contains("orders") && (id.contains("create") || id.contains("preview")) {
        OpName::PlaceOrder
    } else if id.contains("orders") && id.contains("edit") {
        OpName::AmendOrder
    } else if id.contains("orders") && (id.contains("cancel") || id.contains("close")) {
        OpName::CancelOrder
    } else if id.contains("balances") || id.contains("accounts") {
        OpName::FetchBalances
    } else if id.contains("fills") {
        OpName::FetchFills
    } else if id.contains("open-orders") {
        OpName::FetchOpenOrders
    } else if id.contains("positions") {
        OpName::FetchOpenPositions
    } else {
        OpName::FetchStatus
    };
    Ok(op)
}

fn map_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    if id.starts_with("advanced.")
        || id.starts_with("exchange.")
        || id.starts_with("intx.")
        || id.starts_with("other.")
    {
        return map_coinbase_operation_by_id(id);
    }

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
        "crypto.public.ws.trades.trade" => OpName::SubscribeTrades,
        "crypto.private.ws.userdata.executionreport" => OpName::SubscribeExecutionEvents,
        "crypto.public.ws.wsapi.time" => OpName::FetchStatus,
        "crypto.private.ws.wsapi.order.place" => OpName::PlaceOrder,
        "other.public.ws.sbe.marketdata" => OpName::SubscribeOrderbook,
        _ => {
            return Err(UcelError::new(
                ErrorCode::NotSupported,
                format!("unsupported operation mapping for id={id}"),
            ));
        }
    };
    Ok(op)
}

fn map_coinbase_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    let op = if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("candles") || id.contains("klines") {
        OpName::FetchKlines
    } else if id.contains("trade") {
        OpName::FetchTrades
    } else if id.contains("orderbook") || id.contains("book") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("order") && (id.contains("create") || id.contains("place")) {
        OpName::PlaceOrder
    } else if id.contains("order") && id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("balance") || id.contains("account") {
        OpName::FetchBalances
    } else {
        OpName::FetchStatus
    };
    Ok(op)
}

fn normalized_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    if !entry.visibility.trim().is_empty() {
        return Ok(entry.visibility.to_ascii_lowercase());
    }

    if entry.id.contains(".public.") {
        return Ok("public".into());
    }
    if entry.id.contains(".private.") {
        return Ok("private".into());
    }

    Err(UcelError::new(
        ErrorCode::CatalogMissingField,
        format!("missing visibility for id={}", entry.id),
    ))
}

fn map_bybit_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    let id = entry.id.as_str();
    let op = if id.contains(".ws.") {
        if id.contains("ticker") {
            OpName::SubscribeTicker
        } else if id.contains("trade") || id.contains("transaction") {
            OpName::SubscribeTrades
        } else if id.contains("depth") || id.contains("book") {
            OpName::SubscribeOrderbook
        } else if id.contains("position") {
            OpName::SubscribePositionEvents
        } else if id.contains("order") {
            OpName::SubscribeOrderEvents
        } else {
            OpName::FetchStatus
        }
    } else if id.contains("order.create") || id.contains("order.post") || id.contains("add_order") {
        OpName::PlaceOrder
    } else if id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("assets") || id.contains("balance") {
        OpName::FetchBalances
    } else if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("depth") || id.contains("orderbook") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("trade") || id.contains("transactions") {
        OpName::FetchTrades
    } else {
        OpName::FetchStatus
    };
    Ok(op)
}

pub fn default_capabilities(catalog: &ExchangeCatalog) -> Capabilities {
    Capabilities {
        schema_version: "v1".into(),
        kind: "exchange".into(),
        name: catalog.exchange.clone(),
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
    fn maps_all_binance_usdm_ops() {
    fn rejects_duplicate_catalog_ids() {
        let catalog = ExchangeCatalog {
            exchange: "gmo".into(),
            rest_endpoints: vec![CatalogEntry {
                id: "same".into(),
                visibility: Some("public".into()),
                operation: Some("Get ticker".into()),
                method: Some("GET".into()),
                base_url: Some("https://x".into()),
                path: Some("/ok".into()),
                ws_url: None,
                ws: None,
                auth: CatalogAuth {
                    auth_type: "none".into(),
                },
                requires_auth: None,
            }],
            ws_channels: vec![CatalogEntry {
                id: "same".into(),
                visibility: Some("public".into()),
                operation: None,
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://x".into()),
                ws: None,
                auth: CatalogAuth {
                    auth_type: "none".into(),
                },
                requires_auth: None,
            }],
            data_feeds: vec![],
        };
        let err = validate_catalog(&catalog).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogDuplicateId);
    }

    #[test]
    fn rejects_requires_auth_visibility_conflict() {
        let entry = CatalogEntry {
            id: "bybit.public.rest.market.tickers".into(),
            visibility: "public".into(),
            operation: Some("Get Tickers".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.bybit.com".into()),
            path: Some("/v5/market/tickers".into()),
            ws_url: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "api-key+sign".into(),
            },
            requires_auth: Some(true),
        };

        let err = validate_entry(&entry).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }

    #[test]
    fn requires_auth_comes_from_visibility() {
        let private_entry = CatalogEntry {
            id: "crypto.private.ws.executionevents.update".into(),
            visibility: Some("private".into()),
            operation: None,
            method: None,
            base_url: None,
            path: None,
            ws_url: Some("wss://api.coin.z.com/ws/private/v1/xxx".into()),
            ws: None,
            auth: CatalogAuth {
                auth_type: "token".into(),
            },
            requires_auth: None,
        };
        let public_entry = CatalogEntry {
            id: "crypto.public.rest.ticker.get".into(),
            visibility: Some("public".into()),
            operation: Some("Get ticker".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.coin.z.com".into()),
            path: Some("/public/v1/ticker".into()),
            ws_url: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
            requires_auth: None,
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
    fn loads_bybit_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "BYBIT").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 77);
        assert_eq!(catalog.ws_channels.len(), 19);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let op_meta = op_meta_from_entry(entry).unwrap();
            assert_eq!(
                op_meta.requires_auth,
                entry.id.contains(".private."),
                "requires_auth mismatch for {}",
                entry.id
            );
        }
    }
}
