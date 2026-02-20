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
pub type BittradeCatalog = ExchangeCatalog;

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
    pub access: String,
    #[serde(default)]
    pub requires_auth: Option<bool>,
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
            "catalog row id must not be empty",
        ));
    }
    if entry.auth.auth_type.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!(
                "catalog row auth.type must not be empty for id={}",
                entry.id
            ),
        ));
    }

    let visibility = entry_visibility(entry)?;
    if let Some(requires_auth) = entry.requires_auth {
        let expected = visibility == "private";
        if requires_auth != expected {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth conflicts with visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, requires_auth
                ),
            ));
        }
    }

    let ws_url = entry
        .ws_url
        .as_deref()
        .or(entry.ws.as_ref().map(|ws| ws.url.as_str()));

    if let (Some(method), Some(base_url), Some(path)) = (
        entry.method.as_deref(),
        entry.base_url.as_deref(),
        entry.path.as_deref(),
    ) {
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
        return Ok(());
    }

    if let Some(ws_url) = ws_url {
        if ws_url.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("ws endpoint has empty ws_url for id={}", entry.id),
            ));
        }
        if !(ws_url.starts_with("wss://")
            || ws_url.starts_with("ws://")
            || ws_url.starts_with("https://")
            || ws_url.starts_with("http://"))
        {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws_url for id={}: {ws_url}", entry.id),
            ));
        }
        return Ok(());
    }

    Err(UcelError::new(
        ErrorCode::CatalogMissingField,
        format!(
            "catalog row must provide either REST(method/base_url/path) or WS(ws_url/ws.url), id={}",
            entry.id
        ),
    ))
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_operation(entry)?;
    let requires_auth = entry_visibility(entry)? == "private";
    Ok(OpMeta { op, requires_auth })
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    let raw = if !entry.visibility.trim().is_empty() {
        entry.visibility.trim()
    } else if !entry.access.trim().is_empty() {
        entry.access.trim()
    } else if entry.id.contains(".private.") {
        "private"
    } else if entry.id.contains(".public.") {
        "public"
    } else if entry.id.contains(".other.") {
        "other"
    } else {
        ""
    };

    let normalized = raw.to_ascii_lowercase();
    if matches!(normalized.as_str(), "public" | "private" | "other") {
        Ok(normalized)
    } else {
        Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing or invalid visibility/access for id={}", entry.id),
        ))
    }
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }
    Ok(map_operation_by_id(&entry.id))
}

fn map_operation_literal(operation: &str) -> Option<OpName> {
    let op = operation.to_ascii_lowercase();
    if op.contains("ticker") || op.contains("bbo") {
        Some(OpName::FetchTicker)
    } else if op.contains("order book") || op.contains('板') {
        Some(OpName::FetchOrderbookSnapshot)
    } else if op.contains("trade") || op.contains("約定") {
        Some(OpName::FetchTrades)
    } else if op.contains("kline") || op.contains("ローソク") {
        Some(OpName::FetchKlines)
    } else if op.contains("balance")
        || op.contains("残高")
        || op.contains("account")
        || op.contains("口座")
    {
        Some(OpName::FetchBalances)
    } else if op.contains("order") || op.contains("注文") {
        Some(OpName::PlaceOrder)
    } else {
        None
    }
}

fn map_operation_by_id(id: &str) -> OpName {
    if id.contains(".ws.") {
        if id.contains("ticker") || id.contains("bbo") || id.contains("detail") {
            OpName::SubscribeTicker
        } else if id.contains("depth") || id.contains("orderbook") {
            OpName::SubscribeOrderbook
        } else if id.contains("trade") {
            OpName::SubscribeTrades
        } else if id.contains("account") || id.contains("order") || id.contains("execution") {
            OpName::SubscribeExecutionEvents
        } else {
            OpName::FetchStatus
        }
    } else if id.contains("balance") || id.contains("account") {
        OpName::FetchBalances
    } else if id.contains("kline") {
        OpName::FetchKlines
    } else if id.contains("depth") || id.contains("orderbook") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("ticker") || id.contains("bbo") || id.contains("detail") {
        OpName::FetchTicker
    } else if id.contains("trade") {
        OpName::FetchTrades
    } else if id.contains("order") {
        OpName::PlaceOrder
    } else {
        OpName::FetchStatus
    }
}

pub fn capabilities_from_catalog(name: &str, catalog: &ExchangeCatalog) -> Capabilities {
    let has_private = catalog
        .rest_endpoints
        .iter()
        .chain(catalog.ws_channels.iter())
        .any(|entry| {
            entry_visibility(entry)
                .map(|v| v == "private")
                .unwrap_or(false)
        });

    Capabilities {
        schema_version: "1.0.0".into(),
        kind: "capabilities".into(),
        name: name.into(),
        marketdata: MarketDataCapabilities {
            rest: !catalog.rest_endpoints.is_empty(),
            ws: !catalog.ws_channels.is_empty(),
        },
        trading: Some(TradingCapabilities {
            place_order: has_private,
            cancel_order: has_private,
        }),
        auth: Some(AuthCapabilities {
            api_key: has_private,
            passphrase: false,
        }),
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
    use std::path::Path;

    #[test]
    fn loads_bittrade_catalog_and_maps_all_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "bittrade").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 27);
        assert_eq!(catalog.ws_channels.len(), 7);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let op_meta = op_meta_from_entry(entry).unwrap();
            let expected_private =
                entry.id.starts_with("private.") || entry.id.contains(".private.");
            assert_eq!(
                op_meta.requires_auth, expected_private,
                "requires_auth must be mechanically derived from visibility/access for {}",
                entry.id
            );
        }
    }

    #[test]
    fn rejects_duplicate_ids_across_rest_and_ws() {
        let entry = CatalogEntry {
            id: "dup".into(),
            visibility: "public".into(),
            access: String::new(),
            requires_auth: None,
            operation: Some("Get ticker".into()),
            method: Some("GET".into()),
            base_url: Some("https://example.com".into()),
            path: Some("/x".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };

        let catalog = ExchangeCatalog {
            exchange: "x".into(),
            rest_endpoints: vec![entry.clone()],
            ws_channels: vec![CatalogEntry {
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://example.com/ws".into()),
                ..entry
            }],
            data_feeds: vec![],
        };

        let err = validate_catalog(&catalog).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogDuplicateId);
    }
}
