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
    #[serde(default)]
    pub rest_endpoints: Vec<CatalogEntry>,
    #[serde(default)]
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
    pub visibility: Option<String>,
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
    let exchange_dir = exchange.to_ascii_lowercase();
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(exchange_dir)
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
                    "requires_auth conflicts with visibility for id={} (visibility={}, requires_auth={})",
                    entry.id, visibility, requires_auth
                ),
            ));
        }
    }

    let auth_type = entry.auth.auth_type.trim().to_ascii_lowercase();
    if auth_type.is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing auth.type for id={}", entry.id),
        ));
    }
    if visibility == "public" && auth_type != "none" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("public entry requires auth.type=none for id={}", entry.id),
        ));
    }
    if visibility == "private" && auth_type == "none" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!(
                "private entry cannot use auth.type=none for id={}",
                entry.id
            ),
        ));
    }

    let resolved_ws_url = entry
        .ws_url
        .as_deref()
        .or(entry.ws.as_ref().map(|ws| ws.url.as_str()));

    let is_rest = entry.method.is_some() || entry.path.is_some() || entry.base_url.is_some();
    if is_rest {
        let method = entry.method.as_deref().ok_or_else(|| {
            UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("missing method for id={}", entry.id),
            )
        })?;
        let base_url = entry.base_url.as_deref().ok_or_else(|| {
            UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("missing base_url for id={}", entry.id),
            )
        })?;
        let path = entry.path.as_deref().ok_or_else(|| {
            UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("missing path for id={}", entry.id),
            )
        })?;

        if method.trim().is_empty() || base_url.trim().is_empty() || path.trim().is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogMissingField,
                format!("method/base_url/path must not be empty for id={}", entry.id),
            ));
        }
        if !base_url.starts_with("https://") && !base_url.starts_with("http://") {
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

    let ws_url = resolved_ws_url.ok_or_else(|| {
        UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing ws_url for id={}", entry.id),
        )
    })?;
    if ws_url.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("ws_url must not be empty for id={}", entry.id),
        ));
    }
    if !ws_url.starts_with("wss://") && !ws_url.starts_with("ws://") {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid ws_url for id={}: {ws_url}", entry.id),
        ));
    }

    Ok(())
}

pub fn op_meta_from_entry(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_operation(entry)?;
    let requires_auth = entry_visibility(entry)? == "private";
    Ok(OpMeta { op, requires_auth })
}

fn entry_visibility(entry: &CatalogEntry) -> Result<String, UcelError> {
    if let Some(visibility) = entry.visibility.as_deref() {
        let normalized = visibility.trim().to_ascii_lowercase();
        if normalized == "public" || normalized == "private" || normalized == "public/private" {
            return Ok(normalized);
        }
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid visibility for id={}: {visibility}", entry.id),
        ));
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

pub fn map_operation(entry: &CatalogEntry) -> Result<OpName, UcelError> {
    if entry.id.starts_with("coincheck.") {
        return Ok(map_coincheck_operation_by_id(&entry.id));
    }

    if let Some(operation) = entry.operation.as_deref() {
        let op = operation.to_ascii_lowercase();
        if op.contains("ticker") {
            return Ok(OpName::FetchTicker);
        }
        if op.contains("trade") {
            return Ok(OpName::FetchTrades);
        }
        if op.contains("order book") || op.contains("orderbook") || op.contains("depth") {
            return Ok(OpName::FetchOrderbookSnapshot);
        }
        if op.contains("balance") || op.contains("asset") || op.contains("account") {
            return Ok(OpName::FetchBalances);
        }
        if op.contains("cancel") {
            return Ok(OpName::CancelOrder);
        }
        if op.contains("order")
            && (op.contains("create") || op.contains("send") || op.contains("post"))
        {
            return Ok(OpName::PlaceOrder);
        }
    }

    Ok(map_coincheck_operation_by_id(&entry.id))
}

fn map_coincheck_operation_by_id(id: &str) -> OpName {
    match id {
        "coincheck.rest.public.ticker.get" => OpName::FetchTicker,
        "coincheck.rest.public.trades.get" => OpName::FetchTrades,
        "coincheck.rest.public.order_books.get" => OpName::FetchOrderbookSnapshot,
        "coincheck.ws.public.trades" => OpName::SubscribeTrades,
        "coincheck.ws.public.orderbook" => OpName::SubscribeOrderbook,
        "coincheck.ws.private.order_events" | "coincheck.ws.private.execution_events" => {
            OpName::SubscribeExecutionEvents
        }
        "coincheck.rest.private.exchange.orders.post" => OpName::PlaceOrder,
        "coincheck.rest.private.exchange.orders.id.delete" => OpName::CancelOrder,
        "coincheck.rest.private.accounts.balance.get" | "coincheck.rest.private.accounts.get" => {
            OpName::FetchBalances
        }
        _ if id.contains("withdraw")
            || id.contains("bank_accounts")
            || id.contains("send_money") =>
        {
            OpName::FetchBalances
        }
        _ if id.contains("orders") => OpName::FetchOpenOrders,
        _ => OpName::FetchStatus,
    }
}

pub fn capabilities_from_catalog(name: &str, catalog: &ExchangeCatalog) -> Capabilities {
    let mut supports_private = false;
    let mut supports_ws = !catalog.ws_channels.is_empty();

    for entry in catalog
        .rest_endpoints
        .iter()
        .chain(catalog.ws_channels.iter())
    {
        supports_private |= entry.id.contains(".private.");
        supports_ws |= entry.id.contains(".ws.");
    }

    Capabilities {
        schema_version: "1.0.0".into(),
        kind: "capabilities".into(),
        name: name.into(),
        marketdata: MarketDataCapabilities {
            rest: !catalog.rest_endpoints.is_empty(),
            ws: supports_ws,
        },
        trading: Some(TradingCapabilities {
            place_order: supports_private,
            cancel_order: supports_private,
        }),
        auth: Some(AuthCapabilities {
            api_key: supports_private,
            secret: supports_private,
            passphrase: false,
            oauth2: false,
            wallet_signing: false,
        }),
        rate_limit: Some(RateLimitCapabilities {
            has_headers: true,
            retry_after: true,
            key_scoped: true,
            burst_control: true,
        }),
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
    fn loads_coincheck_catalog_and_validates_counts() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "coincheck").unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 25);
        assert_eq!(catalog.ws_channels.len(), 4);
    }

    #[test]
    fn maps_coincheck_catalog_ops_and_derives_requires_auth() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "coincheck").unwrap();

        for entry in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
        {
            let meta = op_meta_from_entry(entry).unwrap();
            assert_eq!(meta.requires_auth, entry.id.contains(".private."));
        }
    }

    #[test]
    fn rejects_duplicate_catalog_ids() {
        let entry = CatalogEntry {
            id: "same".into(),
            visibility: Some("public".into()),
            requires_auth: None,
            operation: Some("ticker".into()),
            method: Some("GET".into()),
            base_url: Some("https://x".into()),
            path: Some("/ok".into()),
            ws_url: None,
            channel: None,
            ws: None,
            auth: CatalogAuth {
                auth_type: "none".into(),
            },
        };
        let catalog = ExchangeCatalog {
            exchange: "dup".into(),
            rest_endpoints: vec![entry.clone()],
            ws_channels: vec![CatalogEntry {
                ws_url: Some("wss://x".into()),
                method: None,
                base_url: None,
                path: None,
                operation: None,
                ..entry
            }],
            data_feeds: vec![],
        };
        let err = validate_catalog(&catalog).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogDuplicateId);
    }
}
