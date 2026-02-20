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
    pub operation: Option<String>,
    pub method: Option<String>,
    pub base_url: Option<String>,
    pub path: Option<String>,
    pub ws_url: Option<String>,
    pub channel: Option<String>,
    #[serde(default)]
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
    if let Some(explicit_requires_auth) = entry.requires_auth {
        let from_visibility = visibility == "private";
        if explicit_requires_auth != from_visibility {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!(
                    "requires_auth conflicts with visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, explicit_requires_auth
                ),
            ));
        }
    }
    Ok(())
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    if !entry.visibility.trim().is_empty() {
        return Ok(entry.visibility.to_ascii_lowercase());
    }
    if entry.id.contains(".private.") {
        return Ok("private".to_string());
    }
    if entry.id.contains(".public.") {
        return Ok("public".to_string());
    }
    Err(UcelError::new(
        ErrorCode::CatalogMissingField,
        format!("missing visibility for id={}", entry.id),
    ))
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    Ok(OpMeta {
        op: map_operation(entry)?,
        requires_auth: entry_visibility(entry)? == "private",
    })
}

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    let id = entry.id.as_str();
    let op = if id.contains("listenkey") {
        OpName::CreateWsAuthToken
    } else if id.contains("orderbooks") || id.contains("depth") || id.contains("orderbook") {
        OpName::SubscribeOrderbook
    } else if id.contains("trade") {
        if id.contains("ws") {
            OpName::SubscribeTrades
        } else {
            OpName::PlaceOrder
        }
    } else if id.contains("kline") {
        OpName::FetchKlines
    } else if id.contains("account") || id.contains("balance") {
        OpName::FetchBalances
    } else if id.contains("execution") || id.contains("userdata") || id.contains("liquidation") {
        OpName::SubscribeExecutionEvents
    } else if id.contains("ticker") || id.contains("markprice") || id.contains("bookticker") {
        OpName::SubscribeTicker
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
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "binance-usdm").unwrap();
        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            assert!(map_operation(entry).is_ok(), "{}", entry.id);
        }
    }

    #[test]
    fn requires_auth_derived_from_visibility() {
        let entry = CatalogEntry {
            id: "x.private.rest.y.get".into(),
            visibility: "private".into(),
            operation: None,
            method: Some("GET".into()),
            base_url: Some("https://x".into()),
            path: Some("/y".into()),
            ws_url: None,
            channel: None,
            requires_auth: None,
            auth: CatalogAuth {
                auth_type: "apiKey".into(),
            },
        };
        assert!(op_meta_from_entry(&entry).unwrap().requires_auth);
    }
}
