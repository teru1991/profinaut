pub mod okx;

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
            "catalog row id must not be empty",
        ));
    }

    let visibility = entry_visibility(entry)?;
    let auth_type = entry.auth.auth_type.trim();
    if auth_type.is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("auth.type is required for id={}", entry.id),
        ));
    }

    let expected_requires_auth = visibility == "private";
    if let Some(requires_auth) = entry.requires_auth {
        if requires_auth != expected_requires_auth {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth contradicts visibility for id={} (visibility={}, requires_auth={})"
            ));
        }
    }

    let rest_shape = entry.method.is_some() || entry.path.is_some() || entry.base_url.is_some();
    let ws_url = entry
        .ws_url
        .as_deref()
        .or(entry.ws.as_ref().map(|ws| ws.url.as_str()));
    let ws_shape = ws_url.is_some()
        || entry
            .base_url
            .as_deref()
            .map(|u| u.starts_with("ws://") || u.starts_with("wss://"))
            .unwrap_or(false);

    match (rest_shape, ws_shape) {
        (true, false) => validate_rest_shape(entry),
        (false, true) => validate_ws_shape(entry, ws_url),
        _ => Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!(
                "catalog row must be either REST(method/base_url/path) or WS(ws_url/ws.url/base_url+channel), id={}",
                entry.id
            ),
        )),
    }
}

fn validate_rest_shape(entry: &CatalogEntry) -> Result<(), UcelError> {
    let method = entry.method.as_deref().unwrap_or_default().trim();
    let base_url = entry.base_url.as_deref().unwrap_or_default().trim();
    let path = entry.path.as_deref().unwrap_or_default().trim();

    if method.is_empty() || base_url.is_empty() || path.is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!(
                "rest endpoint requires method/base_url/path for id={}",
                entry.id
            ),
        ));
    }

    if !method.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid method for id={}: {method}", entry.id),
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

    Ok(())
}

fn validate_ws_shape(entry: &CatalogEntry, ws_url: Option<&str>) -> Result<(), UcelError> {
    if let Some(url) = ws_url {
        let url = url.trim();
        if url.is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("ws endpoint has empty ws_url for id={}", entry.id),
            ));
        }
        if !(url.starts_with("wss://")
            || url.starts_with("ws://")
            || url.starts_with("https://")
            || url.starts_with("http://"))
        {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws_url for id={}: {url}", entry.id),
            ));
        }
    } else {
        let base_url = entry.base_url.as_deref().unwrap_or_default().trim();
        if base_url.is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!(
                    "ws endpoint requires ws_url or base_url for id={}",
                    entry.id
                ),
            ));
        }
        if !(base_url.starts_with("wss://") || base_url.starts_with("ws://")) {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws base_url for id={}: {base_url}", entry.id),
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
                format!("ws endpoint requires channel for id={}", entry.id),
            ));
        }
    }

    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry)? == "private",
    })
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    if entry
        .visibility
        .as_deref()
        .is_some_and(|v| !v.trim().is_empty())
    {
        return Ok(entry
            .visibility
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase());
    }
    if entry.id.contains(".private.") {
        return Ok("private".into());
    }
    if entry.id.contains(".public.") {
        return Ok("public".into());
    }
    Err(UcelError::new(
        ErrorCode::CatalogMissingField,
        format!("missing visibility for id={}", entry.id),
    ))
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if entry.id.starts_with("sbivc.") {
        return map_sbivc_operation(entry);
    }
    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }
    map_operation_by_id(&entry.id)
}

fn map_sbivc_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }

    let op = if entry.id.contains(".ws.") {
        if entry.id.contains("ticker") {
            OpName::SubscribeTicker
        } else if entry.id.contains("trade") {
            OpName::SubscribeTrades
        } else if entry.id.contains("book") || entry.id.contains("orderbook") {
            OpName::SubscribeOrderbook
        } else {
            OpName::FetchStatus
        }
    } else if entry.id.contains("balance") || entry.id.contains("asset") {
        OpName::FetchBalances
    } else if entry.id.contains("order") && entry.id.contains("create") {
        OpName::PlaceOrder
    } else if entry.id.contains("order") && entry.id.contains("cancel") {
        OpName::CancelOrder
    } else if entry.id.contains("order") {
        OpName::FetchOpenOrders
    } else {
        OpName::FetchStatus
    };

    Ok(op)
}

fn map_operation_literal(operation: &str) -> Option<OpName> {
    match operation {
        "Get ticker" | "Get all tickers" => Some(OpName::FetchTicker),
        "Get order book" | "Get depth" => Some(OpName::FetchOrderbookSnapshot),
        "Get recent trades" => Some(OpName::FetchTrades),
        "Get candlesticks" | "Get candlestick" => Some(OpName::FetchKlines),
        "Get account assets" | "Get account balances" => Some(OpName::FetchBalances),
        "Get active orders" => Some(OpName::FetchOpenOrders),
        "Create order" | "Place new order" => Some(OpName::PlaceOrder),
        "Amend order" => Some(OpName::AmendOrder),
        "Cancel order" => Some(OpName::CancelOrder),
        "Get open positions" => Some(OpName::FetchOpenPositions),
        "Get position summary" => Some(OpName::FetchPositionSummary),
        "stream subscribe" => Some(OpName::CreateWsAuthToken),
        _ => None,
    }
}

fn map_operation_by_id(id: &str) -> Result<OpName, UcelError> {
    let op = if id.contains(".ws.") {
        if id.contains("ticker") {
            OpName::SubscribeTicker
        } else if id.contains("trade") {
            OpName::SubscribeTrades
        } else if id.contains("book") || id.contains("depth") {
            OpName::SubscribeOrderbook
        } else if id.contains("execution") {
            OpName::SubscribeExecutionEvents
        } else {
            OpName::FetchStatus
        }
    } else if id.contains("order") && id.contains("create") {
        OpName::PlaceOrder
    } else if id.contains("order") && id.contains("amend") {
        OpName::AmendOrder
    } else if id.contains("order") && id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("balance") || id.contains("asset") || id.contains("account") {
        OpName::FetchBalances
    } else if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("trade") {
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
    use std::path::Path;

    #[test]
    fn loads_upbit_catalog_and_maps_all_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "upbit").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 22);
        assert_eq!(catalog.ws_channels.len(), 7);

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let op_meta = op_meta_from_entry(entry).unwrap();
            assert_eq!(
                op_meta.requires_auth,
                entry.visibility.eq_ignore_ascii_case("private")
            );
            assert!(map_operation(entry).is_ok());
        }
    }

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let catalog = ExchangeCatalog {
            exchange: "x".into(),
            rest_endpoints: vec![CatalogEntry {
                id: "same".into(),
                visibility: "public".into(),
                requires_auth: Some(false),
                operation: Some("Get ticker".into()),
                method: Some("GET".into()),
                base_url: Some("https://api.example.com".into()),
                path: Some("/ticker".into()),
                ws_url: None,
                ws: None,
                auth: CatalogAuth {
                    auth_type: "none".into(),
                },
            }],
            ws_channels: vec![CatalogEntry {
                id: "same".into(),
                visibility: "public".into(),
                requires_auth: Some(false),
                operation: None,
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://stream.example.com".into()),
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
    fn sbivc_requires_auth_is_mechanical_from_visibility() {
        let private = CatalogEntry {
            id: "sbivc.private.rest.account.balance".into(),
            visibility: "private".into(),
            requires_auth: Some(true),
            operation: Some("Get account balances".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.sbivc.co.jp".into()),
            path: Some("/v1/account/balance".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "api-key+sign".into(),
            },
        };
        let meta = op_meta_from_entry(&private).unwrap();
        assert_eq!(meta.op, OpName::FetchBalances);
        assert!(meta.requires_auth);

        let bad = CatalogEntry {
            requires_auth: Some(false),
            ..private
        };
        let err = validate_catalog(&ExchangeCatalog {
            exchange: "sbivc".into(),
            rest_endpoints: vec![bad],
            ws_channels: vec![],
            data_feeds: vec![],
        })
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }

    #[test]
    fn loads_sbivc_catalog_from_repo() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "sbivc").unwrap();
        assert_eq!(catalog.exchange, "sbivc");
        assert_eq!(catalog.rest_endpoints.len(), 0);
        assert_eq!(catalog.ws_channels.len(), 0);
    }
}
