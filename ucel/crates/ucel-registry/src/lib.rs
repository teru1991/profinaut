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
}

pub type GmoCatalog = ExchangeCatalog;
pub type BitbankCatalog = ExchangeCatalog;

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
    pub channel: Option<String>,
    pub ws: Option<CatalogWs>,
    pub auth: CatalogAuth,
    pub requires_auth: Option<bool>,
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
            return Err(UcelError::new(
                ErrorCode::CatalogDuplicateId,
                format!("duplicate id {}", e.id),
            ));
        }
    }
    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry) == "private",
    })
}

fn entry_visibility(entry: &CatalogEntry) -> String {
    if !entry.visibility.is_empty() {
        return entry.visibility.to_ascii_lowercase();
    }
    if entry.id.contains(".private.") {
        "private".into()
    } else {
        "public".into()
    }
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
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
