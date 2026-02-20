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
    #[serde(default)]
    pub requires_auth: Option<bool>,
    #[serde(default)]
    pub channel: Option<String>,
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
            return Err(UcelError::new(
                ErrorCode::CatalogDuplicateId,
                format!("duplicate catalog id={}", e.id),
            ));
        }
        validate_entry(e)?;
    }
    Ok(())
}

fn validate_entry(entry: &CatalogEntry) -> Result<(), UcelError> {
    if entry.id.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "catalog row has empty id",
        ));
    }

    let visibility = entry_visibility(entry)?;
    if visibility != "public" && visibility != "private" && visibility != "public/private" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!(
                "invalid visibility={} for id={}",
                entry.visibility, entry.id
            ),
        ));
    }

    if let Some(requires_auth) = entry.requires_auth {
        let expected_requires_auth = visibility == "private";
        if requires_auth != expected_requires_auth {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth contradicts visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, requires_auth
                ),
            ));
        }
    }

    let ws_url = entry
        .ws_url
        .as_deref()
        .or_else(|| entry.ws.as_ref().map(|ws| ws.url.as_str()));

    if let (Some(method), Some(base_url), Some(path)) = (
        entry.method.as_deref(),
        entry.base_url.as_deref(),
        entry.path.as_deref(),
    ) {
        if method.trim().is_empty() || base_url.trim().is_empty() || path.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("empty method/base_url/path for id={}", entry.id),
            ));
        }
        if !method.chars().all(|ch| ch.is_ascii_uppercase()) {
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
        return Ok(());
    }

    if let Some(url) = ws_url {
        if url.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("empty ws_url for id={}", entry.id),
            ));
        }
        if !(url.starts_with("wss://") || url.starts_with("ws://")) {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws_url for id={}: {url}", entry.id),
            ));
        }
        return Ok(());
    }

    Err(UcelError::new(
        ErrorCode::CatalogMissingField,
        format!(
            "catalog row must define REST(method/base_url/path) or WS(ws_url/ws.url), id={}",
            entry.id
        ),
    ))
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry)? == "private",
    })
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    if !entry.visibility.trim().is_empty() {
        return Ok(entry.visibility.to_ascii_lowercase());
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
    if entry.id.starts_with("quotation.")
        || entry.id.starts_with("exchange.")
        || entry.id.starts_with("other.")
    {
        return map_upbit_operation_from_id(&entry.id);
    }

    Ok(map_operation_fallback(&entry.id))
}

fn map_upbit_operation_from_id(id: &str) -> Result<OpName, UcelError> {
    let op = if id.contains(".ws.") {
        if id.contains("ticker") {
            OpName::SubscribeTicker
        } else if id.contains("trade") {
            OpName::SubscribeTrades
        } else if id.contains("orderbook") {
            OpName::SubscribeOrderbook
        } else if id.contains("myorder") {
            OpName::SubscribeOrderEvents
        } else if id.contains("myasset") {
            OpName::SubscribePositionEvents
        } else {
            OpName::FetchStatus
        }
    } else if id.contains("orders.create") {
        OpName::PlaceOrder
    } else if id.contains("orders.cancel") {
        OpName::CancelOrder
    } else if id.contains("orders.open") || id.contains("orders.closed") {
        OpName::FetchOpenOrders
    } else if id.contains("accounts") {
        OpName::FetchBalances
    } else if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("trades") {
        OpName::FetchTrades
    } else if id.contains("orderbook") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("candles") {
        OpName::FetchKlines
    } else {
        OpName::FetchStatus
    };

    if id.trim().is_empty() {
        return Err(UcelError::new(ErrorCode::NotSupported, "empty id"));
    }

    Ok(op)
}

fn map_operation_fallback(id: &str) -> OpName {
    if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("trade") {
        OpName::FetchTrades
    } else if id.contains("orderbook") || id.contains("depth") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("kline") || id.contains("candle") {
        OpName::FetchKlines
    } else if id.contains("order") && id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("order") && (id.contains("create") || id.contains("post")) {
        OpName::PlaceOrder
    } else {
        OpName::FetchStatus
    }
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
                requires_auth: None,
                channel: None,
                operation: None,
                method: Some("GET".into()),
                base_url: Some("https://api.x".into()),
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
                requires_auth: None,
                operation: None,
                method: None,
                base_url: None,
                path: None,
                ws_url: Some("wss://api.x/ws".into()),
                channel: Some("ticker".into()),
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
}
