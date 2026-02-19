use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use ucel_core::{
    default_requires_auth, AuthCapabilities, Capabilities, ErrorCode, FailoverPolicy, MarketDataCapabilities,
    OpMeta, OpName, OperationalCapabilities, RateLimitCapabilities, RuntimePolicy, SafeDefaults,
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
pub struct CatalogEntry {
    pub id: String,
    pub service: String,
    pub visibility: String,
    pub operation: Option<String>,
    pub source_url: Option<String>,
    pub rate_limit_cost: Option<u32>,
}

pub fn load_catalog_from_path(path: &Path) -> Result<Vec<CatalogEntry>, UcelError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        UcelError::new(
            ErrorCode::RegistryInvalidCatalog,
            format!("failed to read {}: {e}", path.display()),
        )
    })?;
    let entries: Vec<CatalogEntry> = serde_json::from_str(&raw).map_err(|e| {
        UcelError::new(
            ErrorCode::RegistryInvalidCatalog,
            format!("failed to parse {}: {e}", path.display()),
        )
    })?;
    validate_catalog(&entries)?;
    Ok(entries)
}

pub fn load_catalog_from_repo_root(repo_root: &Path, exchange: &str) -> Result<Vec<CatalogEntry>, UcelError> {
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(exchange)
        .join("catalog.json");
    load_catalog_from_path(&path)
}

pub fn validate_catalog(entries: &[CatalogEntry]) -> Result<(), UcelError> {
    let mut seen = HashSet::new();
    for e in entries {
        if e.id.trim().is_empty() || e.service.trim().is_empty() || e.visibility.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::RegistryInvalidCatalog,
                "catalog entry missing required keys",
            ));
        }
        if !seen.insert(e.id.clone()) {
            return Err(UcelError::new(
                ErrorCode::RegistryInvalidCatalog,
                format!("duplicate id found: {}", e.id),
            ));
        }
    }
    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_operation(entry)?;
    let requires_auth = entry.visibility.eq_ignore_ascii_case("private") || default_requires_auth(op);
    Ok(OpMeta { op, requires_auth })
}

fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    let source = entry
        .operation
        .as_deref()
        .unwrap_or(entry.id.as_str())
        .to_ascii_lowercase();
    let op = match source.as_str() {
        "fetch_ticker" | "ticker" => OpName::FetchTicker,
        "fetch_trades" | "trades" => OpName::FetchTrades,
        "fetch_orderbook_snapshot" | "orderbook_snapshot" => OpName::FetchOrderbookSnapshot,
        "subscribe_ticker" => OpName::SubscribeTicker,
        "subscribe_trades" => OpName::SubscribeTrades,
        "subscribe_orderbook" => OpName::SubscribeOrderbook,
        "place_order" => OpName::PlaceOrder,
        "cancel_order" => OpName::CancelOrder,
        "fetch_balances" => OpName::FetchBalances,
        "fetch_open_orders" => OpName::FetchOpenOrders,
        "fetch_fills" => OpName::FetchFills,
        _ => {
            return Err(UcelError::new(
                ErrorCode::NotSupported,
                format!("unsupported operation mapping for id={}", entry.id),
            ))
        }
    };
    Ok(op)
}

pub fn capabilities_from_catalog(name: &str, entries: &[CatalogEntry]) -> Capabilities {
    let mut has_rest = false;
    let mut has_ws = false;
    for e in entries {
        match e.service.to_ascii_lowercase().as_str() {
            "rest" => has_rest = true,
            "ws" => has_ws = true,
            _ => {}
        }
    }
    Capabilities {
        schema_version: "1.1.4".into(),
        kind: "exchange".into(),
        name: name.into(),
        marketdata: MarketDataCapabilities { rest: has_rest, ws: has_ws },
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
        allowed_ops: vec![OpName::FetchTicker, OpName::FetchTrades, OpName::FetchOrderbookSnapshot],
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
    use std::{env, fs};

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let entries = vec![
            CatalogEntry {
                id: "same".into(),
                service: "rest".into(),
                visibility: "public".into(),
                operation: Some("fetch_ticker".into()),
                source_url: None,
                rate_limit_cost: None,
            },
            CatalogEntry {
                id: "same".into(),
                service: "ws".into(),
                visibility: "public".into(),
                operation: Some("subscribe_ticker".into()),
                source_url: None,
                rate_limit_cost: None,
            },
        ];
        let err = validate_catalog(&entries).unwrap_err();
        assert_eq!(err.code, ErrorCode::RegistryInvalidCatalog);
    }

    #[test]
    fn derives_requires_auth_from_visibility() {
        let entry = CatalogEntry {
            id: "po".into(),
            service: "rest".into(),
            visibility: "private".into(),
            operation: Some("place_order".into()),
            source_url: None,
            rate_limit_cost: None,
        };
        let meta = op_meta_from_entry(&entry).unwrap();
        assert_eq!(meta.op, OpName::PlaceOrder);
        assert!(meta.requires_auth);
    }

    #[test]
    fn loads_catalog_json_file() {
        let tmp = env::temp_dir().join("ucel_registry_catalog_test.json");
        fs::write(
            &tmp,
            r#"[{"id":"ticker","service":"rest","visibility":"public","operation":"fetch_ticker"}]"#,
        )
        .unwrap();
        let entries = load_catalog_from_path(&tmp).unwrap();
        assert_eq!(entries.len(), 1);
        fs::remove_file(tmp).unwrap();
    }
}
