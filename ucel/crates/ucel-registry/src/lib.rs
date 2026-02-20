pub mod deribit;
pub mod okx;
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
    #[serde(default)]
    pub rest_endpoints: Vec<CatalogEntry>,
    #[serde(default)]
    pub ws_channels: Vec<CatalogEntry>,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct CatalogAuth {
    #[serde(rename = "type", default)]
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
    let exchange_dir = exchange.to_ascii_lowercase();
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(&exchange_dir)
        .join("catalog.json");

    if exchange_dir == "deribit" {
        return deribit::load_deribit_catalog_from_path(&path);
    }

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
                ErrorCode::CatalogDuplicateId,
                format!("duplicate id={}", e.id),
            ));
        }
    }
    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry)?.eq("private"),
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
    Ok(map_operation_fallback(&entry.id))
}

fn map_operation_fallback(id: &str) -> OpName {
    if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("trade") {
        OpName::FetchTrades
    } else if id.contains("kline") || id.contains("candle") {
        OpName::FetchKlines
    } else if id.contains("orderbook") || id.contains("depth") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("balance") {
        OpName::FetchBalances
    } else if id.contains("order") && id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("order") {
        OpName::PlaceOrder
    } else {
        OpName::FetchStatus
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::path::Path;

    #[derive(Debug, Deserialize)]
    struct CoverageManifest {
        venue: String,
        strict: bool,
        entries: Vec<CoverageEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct CoverageEntry {
        id: String,
        implemented: bool,
        tested: bool,
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

        let uncovered: Vec<_> = manifest
            .entries
            .iter()
            .filter(|entry| !entry.implemented || !entry.tested)
            .map(|entry| entry.id.clone())
            .collect();
        assert!(
            uncovered.is_empty(),
            "strict gate requires zero gaps: {uncovered:?}"
        );
    }
}
