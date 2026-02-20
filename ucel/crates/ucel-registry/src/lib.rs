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
    pub channel: Option<String>,
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
            if auth_type_requires_credentials(&entry.auth.auth_type) {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!(
                        "public endpoint must not require credentials for id={} (auth.type={})",
                        entry.id, entry.auth.auth_type
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
    let ws_base_url = entry
        .base_url
        .as_deref()
        .filter(|base_url| base_url.starts_with("wss://") || base_url.starts_with("ws://"));
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
            if !method
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch == '_' || ch == '-' || ch == '/')
            {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid method for id={}: {method}", entry.id),
                ));
            }
            let is_doc_ref =
                entry.operation.as_deref() == Some("doc-ref") || entry.id.ends_with(".ref");
            if !(base_url.starts_with("https://")
                || base_url.starts_with("http://")
                || (is_doc_ref && base_url.starts_with("docs://")))
            {
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
        (None, _, None, Some(ws_url)) => {
            if ws_url.trim().is_empty() {
                return Err(UcelError::new(
                    ErrorCode::CatalogMissingField,
                    format!("ws endpoint has empty ws_url for id={}", entry.id),
                ));
            }
            let is_non_socket_reference = entry.id == "other.public.ws.sbe.marketdata";
            if !(ws_url.starts_with("wss://")
                || ws_url.starts_with("ws://")
                || (is_non_socket_reference && ws_url.contains("see docs")))
            {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid ws_url for id={}: {ws_url}", entry.id),
                ));
            }
        }
        (None, Some(base_url), None, None) if ws_base_url.is_some() => {
            if base_url.trim().is_empty() {
                return Err(UcelError::new(
                    ErrorCode::CatalogMissingField,
                    format!("ws endpoint has empty base_url for id={}", entry.id),
                ));
            }
            if entry
                .channel
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(UcelError::new(
                    ErrorCode::CatalogMissingField,
                    format!("ws endpoint requires non-empty channel for id={}", entry.id),
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

fn auth_type_requires_credentials(auth_type: &str) -> bool {
    matches!(auth_type, "apiKey" | "token" | "oauth2")
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
    map_operation_by_id(&entry.id)
}

fn map_operation_literal(operation: &str) -> Option<OpName> {
    match operation {
        "Get service status" | "Get FX API status" | "List futures instruments" => {
            Some(OpName::FetchStatus)
        }
        "Test connectivity"
        | "Get server time"
        | "Exchange/symbol metadata"
        | "Start user data stream (legacy)"
        | "doc-ref" => Some(OpName::FetchStatus),
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

fn map_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    let op = match id {
        "options.public.rest.general.ref"
        | "options.public.rest.errors.ref"
        | "options.public.rest.market.ref" => OpName::FetchStatus,
        "options.private.rest.trade.ref" => OpName::PlaceOrder,
        "options.private.rest.account.ref" => OpName::FetchBalances,
        "options.private.rest.listenkey.post" => OpName::CreateWsAuthToken,
        "options.private.rest.listenkey.put" => OpName::ExtendWsAuthToken,
        "options.private.rest.listenkey.delete" => OpName::CancelOrder,
        "options.public.ws.trade" => OpName::SubscribeTrades,
        "options.public.ws.ticker"
        | "options.public.ws.markprice"
        | "options.public.ws.indexprice" => OpName::SubscribeTicker,
        "options.public.ws.depth" => OpName::SubscribeOrderbook,
        "options.public.ws.kline" => OpName::FetchKlines,
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
                channel: None,
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
                channel: None,
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
            channel: None,
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
            channel: None,
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
    fn loads_binance_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "binance").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 9);
        assert_eq!(catalog.ws_channels.len(), 5);

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
    fn loads_binance_options_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "binance-options").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 8);
        assert_eq!(catalog.ws_channels.len(), 6);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let op_meta = op_meta_from_entry(entry).unwrap();
            assert_eq!(
                op_meta.requires_auth,
                entry.visibility.eq_ignore_ascii_case("private"),
                "requires_auth must be derived from visibility for {}",
                entry.id
            );
        }
    }
}
