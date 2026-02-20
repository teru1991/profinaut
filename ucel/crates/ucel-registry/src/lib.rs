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
pub type BitbankCatalog = ExchangeCatalog;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DataFeedEntry {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    #[serde(default)]
    pub visibility: String,
    #[serde(default)]
    pub requires_auth: Option<bool>,
    pub operation: Option<String>,
    pub method: Option<String>,
    pub base_url: Option<String>,
    pub path: Option<String>,
    pub ws_url: Option<String>,
    pub channel: Option<String>,
    pub ws: Option<CatalogWs>,
    #[serde(default)]
    pub auth: CatalogAuth,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CatalogWs {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct CatalogAuth {
    #[serde(rename = "type", default)]
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
        .join(exchange.to_ascii_lowercase())
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
    if entry.id.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "catalog entry id must not be empty",
        ));
    }

    let visibility = entry_visibility(entry)?;
    if let Some(requires_auth) = entry.requires_auth {
        let expected = visibility == "private";
        if requires_auth != expected {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth contradicts visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, requires_auth
                ),
            ));
        }
    }

    if entry.auth.auth_type.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing auth.type for id={}", entry.id),
        ));
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
                        "rest row has empty method/base_url/path for id={}",
                        entry.id
                    ),
                ));
            }
            if !(method.chars().all(|c| c.is_ascii_uppercase()) || is_placeholder(method)) {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid method for id={}: {method}", entry.id),
                ));
            }
            if !is_valid_httpish_url(base_url) {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid base_url for id={}: {base_url}", entry.id),
                ));
            }
            if !(path.starts_with('/') || is_placeholder(path)) {
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
                    format!("ws row has empty ws_url for id={}", entry.id),
                ));
            }
            if !is_valid_wsish_url(ws_url) {
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
                    "catalog row must be either REST(method/base_url/path) or WS(ws_url/ws.url), id={}",
                    entry.id
                ),
            ));
        }
    }

    Ok(())
}

fn is_placeholder(value: &str) -> bool {
    let v = value.trim().to_ascii_lowercase();
    matches!(
        v.as_str(),
        "unknown" | "not_applicable" | "n/a" | "na" | "tbd" | "todo"
    ) || (v.starts_with("{{") && v.ends_with("}}"))
        || (v.starts_with("${") && v.ends_with('}'))
}

fn is_valid_httpish_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://") || is_placeholder(url)
}

fn is_valid_wsish_url(url: &str) -> bool {
    url.starts_with("wss://")
        || url.starts_with("ws://")
        || url.starts_with("https://")
        || url.starts_with("http://")
        || is_placeholder(url)
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    let normalized = if !entry.visibility.trim().is_empty() {
        entry.visibility.to_ascii_lowercase()
    } else if entry.id.contains(".private.") {
        "private".to_string()
    } else if entry.id.contains(".public.") {
        "public".to_string()
    } else {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing visibility for id={}", entry.id),
        ));
    };

    match normalized.as_str() {
        "public" | "private" | "public/private" => Ok(normalized),
        _ => Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid visibility={} for id={}", normalized, entry.id),
        )),
    }
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry)? == "private",
    })
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if let Some(op) = map_operation_literal(entry.operation.as_deref().unwrap_or("")) {
        return Ok(op);
    }

    if entry.id.contains(".ws.") {
        return Ok(map_ws_operation_by_id(&entry.id));
    }

    Ok(map_rest_operation_by_id(&entry.id))
}

fn map_operation_literal(operation: &str) -> Option<OpName> {
    let op = operation.to_ascii_lowercase();
    if op.contains("ticker") {
        Some(OpName::FetchTicker)
    } else if op.contains("trade") {
        Some(OpName::FetchTrades)
    } else if op.contains("kline") || op.contains("candle") {
        Some(OpName::FetchKlines)
    } else if op.contains("order book") || op.contains("orderbook") || op.contains("depth") {
        Some(OpName::FetchOrderbookSnapshot)
    } else if op.contains("balance") || op.contains("account") {
        Some(OpName::FetchBalances)
    } else if op.contains("status") || op.contains("catalog") {
        Some(OpName::FetchStatus)
    } else {
        None
    }
}

fn map_ws_operation_by_id(id: &str) -> OpName {
    let id = id.to_ascii_lowercase();
    if id.contains("ticker") {
        OpName::SubscribeTicker
    } else if id.contains("trade") {
        OpName::SubscribeTrades
    } else if id.contains("book") || id.contains("depth") {
        OpName::SubscribeOrderbook
    } else if id.contains("execution") || id.contains("fills") {
        OpName::SubscribeExecutionEvents
    } else if id.contains("position") {
        OpName::SubscribePositionEvents
    } else if id.contains("order") {
        OpName::SubscribeOrderEvents
    } else {
        OpName::FetchStatus
    }
}

fn map_rest_operation_by_id(id: &str) -> OpName {
    let id = id.to_ascii_lowercase();
    if id.contains("place") || id.contains("create") || id.contains("send") {
        OpName::PlaceOrder
    } else if id.contains("amend") || id.contains("edit") {
        OpName::AmendOrder
    } else if id.contains("cancel") || id.contains("close") {
        OpName::CancelOrder
    } else if id.contains("balance") || id.contains("account") || id.contains("wallet") {
        OpName::FetchBalances
    } else if id.contains("open-order") || id.contains("orders") {
        OpName::FetchOpenOrders
    } else if id.contains("position") {
        OpName::FetchOpenPositions
    } else if id.contains("trade") {
        OpName::FetchTrades
    } else if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("book") || id.contains("depth") {
        OpName::FetchOrderbookSnapshot
    } else {
        OpName::FetchStatus
    }
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
    fn loads_htx_catalog_and_maps_all_ops() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "htx").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 13);
        assert_eq!(catalog.ws_channels.len(), 9);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let op_meta = op_meta_from_entry(entry).unwrap();
            assert_eq!(op_meta.requires_auth, entry.visibility == "private");
        }
    }

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let base = CatalogEntry {
            id: "same.id".into(),
            visibility: "public".into(),
            requires_auth: None,
            operation: Some("status".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.example.com".into()),
            path: Some("/v1/status".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };
        let catalog = ExchangeCatalog {
            exchange: "x".into(),
            rest_endpoints: vec![base.clone()],
            ws_channels: vec![CatalogEntry {
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://api.example.com/ws".into()),
                ..base
            }],
            data_feeds: vec![],
        };

        let err = validate_catalog(&catalog).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogDuplicateId);
    }

    #[test]
    fn rejects_requires_auth_contradiction() {
        let entry = CatalogEntry {
            id: "x.private.rest.balance".into(),
            visibility: "private".into(),
            requires_auth: Some(false),
            operation: Some("balance".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.example.com".into()),
            path: Some("/v1/balance".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "signed".into(),
            },
        };

        let err = validate_entry(&entry).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }
}
