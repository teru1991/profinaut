pub mod deribit;
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
    #[serde(default)]
    pub rest_endpoints: Vec<CatalogEntry>,
    #[serde(default)]
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
    pub visibility: Option<String>,
    #[serde(default)]
    pub access: String,
    #[serde(default)]
    pub requires_auth: Option<bool>,
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
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(exchange.to_ascii_lowercase())
        .join("catalog.json");

    if exchange_dir == "deribit" {
        return deribit::load_deribit_catalog_from_path(&path);
    }

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
    if entry.auth.auth_type.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!(
                "catalog row auth.type must not be empty for id={}",
                entry.id
            ),
        ));
    }
    if entry.visibility.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing visibility for id={}", entry.id),
        ));
    }
    if entry.auth.auth_type.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing auth.type for id={}", entry.id),
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
                || base_url.starts_with("docs://"))
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
        (None, _, None, Some(ws_url)) => {
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
    if entry.id.starts_with("jsonrpc.") || entry.id.starts_with("ws.") {
        return deribit::map_deribit_operation(entry);
    }

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
    if entry.id.starts_with("usdm.") {
        return map_binance_usdm_operation(entry);
    }
    if entry.id.starts_with("bybit.") {
        return map_bybit_operation(entry);
    }
    if let Some(op) = map_bitget_operation_by_id(&entry.id) {
        return Ok(op);
    }

    if let Some(operation) = entry.operation.as_deref() {
        if let Some(op) = map_operation_literal(operation) {
            return Ok(op);
        }
    }
    Ok(map_operation_by_id(&entry.id))
}

fn map_bitget_operation_by_id(id: &str) -> Option<OpName> {
    match id {
        "other.public.rest.nav.blocked" | "other.public.ws.nav.blocked" => {
            Some(OpName::FetchStatus)
        }
        _ => None,
    }
}

fn map_binance_usdm_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    let op = match entry.id.as_str() {
        "usdm.public.rest.general.ref"
        | "usdm.public.rest.errors.ref"
        | "usdm.public.rest.market.ref"
        | "usdm.public.ws.wsapi.general" => OpName::FetchStatus,
        "usdm.private.rest.trade.ref" | "usdm.private.rest.listenkey.ref" => OpName::PlaceOrder,
        "usdm.private.rest.account.ref" => OpName::FetchBalances,
        "usdm.public.ws.market.root"
        | "usdm.public.ws.market.markprice"
        | "usdm.public.ws.market.bookticker" => OpName::SubscribeTicker,
        "usdm.public.ws.market.aggtrade" => OpName::SubscribeTrades,
        "usdm.public.ws.market.kline" => OpName::FetchKlines,
        "usdm.public.ws.market.depth.partial" | "usdm.public.ws.market.depth.diff" => {
            OpName::SubscribeOrderbook
        }
        "usdm.public.ws.market.liquidation" | "usdm.private.ws.userdata.events" => {
            OpName::SubscribeExecutionEvents
        }
        _ => {
            return Err(UcelError::new(
                ErrorCode::NotSupported,
                format!(
                    "unsupported binance-usdm operation mapping for id={}",
                    entry.id
                ),
            ));
        }
    };
    Ok(op)
}

fn map_bitmex_operation_by_id(id: &str) -> Option<OpName> {
    if let Some(channel) = id
        .strip_prefix("public.ws.")
        .and_then(|raw| raw.strip_suffix(".subscribe"))
    {
        return Some(map_bitmex_public_ws_channel(channel));
    }

    if let Some(channel) = id
        .strip_prefix("private.ws.")
        .and_then(|raw| raw.strip_suffix(".subscribe"))
    {
        Some(OpName::FetchBalances)
    } else if op.contains("order") || op.contains("注文") {
        Some(OpName::PlaceOrder)
    } else {
        None
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
            ws: supports_ws,
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
    fn rejects_duplicate_catalog_ids() {
        let catalog = ExchangeCatalog {
            exchange: "gmo".into(),
            rest_endpoints: vec![CatalogEntry {
                id: "same".into(),
                visibility: "public".into(),
                requires_auth: None,
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
                requires_auth: None,
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
    fn rejects_requires_auth_visibility_conflict() {
        let entry = CatalogEntry {
            id: "bybit.public.rest.market.tickers".into(),
            visibility: "public".into(),
            requires_auth: Some(true),
            operation: Some("Get Tickers".into()),
            method: Some("GET".into()),
            base_url: Some("https://api.bybit.com".into()),
            path: Some("/v5/market/tickers".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "api-key+sign".into(),
            },
        };
        let err = validate_entry(&entry).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }

    #[test]
    fn requires_auth_comes_from_visibility() {
        let private_entry = CatalogEntry {
            id: "crypto.private.ws.executionevents.update".into(),
            visibility: "private".into(),
            requires_auth: None,
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
            requires_auth: None,
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
    fn rejects_requires_auth_contradiction() {
        let entry = CatalogEntry {
            id: "x.private.rest.y.get".into(),
            visibility: "private".into(),
            requires_auth: Some(false),
            operation: Some("Get ticker".into()),
            method: Some("GET".into()),
            base_url: Some("https://x".into()),
            path: Some("/ok".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "apiKey".into(),
            },
        };

        let err = validate_entry(&entry).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }

    #[test]
    fn loads_gmocoin_catalog() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "gmocoin").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 30);
        assert_eq!(catalog.ws_channels.len(), 12);
    }

    #[test]
    fn bitget_catalog_fail_fast_on_invalid_urls() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let err = load_catalog_from_repo_root(&repo_root, "bitget").unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }

    #[test]
    fn map_operation_handles_bitget_placeholder_ids() {
        let rest = CatalogEntry {
            id: "other.public.rest.nav.blocked".into(),
            visibility: "public".into(),
            requires_auth: Some(false),
            operation: Some("not_applicable_due_to_source_access_failure".into()),
            method: Some("GET".into()),
            base_url: Some("https://docs.bitget.com".into()),
            path: Some("/placeholder".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };
        let ws = CatalogEntry {
            id: "other.public.ws.nav.blocked".into(),
            visibility: "public".into(),
            requires_auth: Some(false),
            operation: None,
            method: None,
            base_url: None,
            path: None,
            ws_url: Some("wss://docs.bitget.com/ws".into()),
            channel: Some("common".into()),
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };

        assert!(matches!(
            op_meta_from_entry(&rest).unwrap().op,
            OpName::FetchStatus
        ));
        assert!(matches!(
            op_meta_from_entry(&ws).unwrap().op,
            OpName::FetchStatus
        ));
    }
}
