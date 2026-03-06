use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use ucel_core::{
    normalize_reject_class, retry_safety_for, PrivateRestOperation, PrivateRestSupport,
    RetrySafety, VenueAccessScope, VenueRejectClass,
};
use ucel_registry::hub::ExchangeId;
use ucel_registry::{default_capabilities_for_exchange, load_catalog_from_repo_root};

#[derive(Debug, Deserialize)]
pub struct FixtureCanonical {
    pub balances: Vec<FixtureBalance>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

pub fn fixture_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ucel").join("fixtures").join("private_rest")
}

pub fn has_required_fixtures(repo_root: &Path, venue: &str) -> bool {
    let base = fixture_root(repo_root).join(venue);
    base.join("request_preview.json").exists()
        && base.join("raw_response.json").exists()
        && base.join("expected_canonical.json").exists()
}

pub fn load_expected_canonical(repo_root: &Path, venue: &str) -> Result<FixtureCanonical, String> {
    let p = fixture_root(repo_root)
        .join(venue)
        .join("expected_canonical.json");
    let body = fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("parse {}: {e}", p.display()))
}

pub fn support_for(
    exchange: ExchangeId,
    op: PrivateRestOperation,
    repo_root: &Path,
) -> PrivateRestSupport {
    let scope = default_capabilities_for_exchange(exchange)
        .ok()
        .and_then(|c| c.venue_access.map(|v| v.scope))
        .unwrap_or(VenueAccessScope::PublicOnly);

    if !matches!(scope, VenueAccessScope::PublicPrivate) {
        return PrivateRestSupport::BlockedByPolicy;
    }

    let catalog = match load_catalog_from_repo_root(repo_root, exchange.as_str()) {
        Ok(c) => c,
        Err(_) => return PrivateRestSupport::NotSupported,
    };

    let ids: Vec<String> = catalog
        .rest_endpoints
        .iter()
        .map(|e| e.id.to_ascii_lowercase())
        .collect();

    let present = match op {
        PrivateRestOperation::GetBalances => ids
            .iter()
            .any(|id| id.contains("balance") || id.contains("assets")),
        PrivateRestOperation::GetOpenOrders => ids
            .iter()
            .any(|id| id.contains("openorders") || id.contains("open_orders")),
        PrivateRestOperation::GetOrder => ids
            .iter()
            .any(|id| id.contains("order.get") || id.contains("order.query")),
        PrivateRestOperation::CancelOrder => ids.iter().any(|id| id.contains("cancel")),
        PrivateRestOperation::GetFills => ids.iter().any(|id| {
            id.contains("fills") || id.contains("matchresults") || id.contains("executions")
        }),
        PrivateRestOperation::GetAccountProfile => ids
            .iter()
            .any(|id| id.contains("account") || id.contains("profile")),
        PrivateRestOperation::GetPositions => ids.iter().any(|id| id.contains("position")),
    };

    if present {
        PrivateRestSupport::Supported
    } else {
        PrivateRestSupport::Partial
    }
}

pub fn normalize_reason(
    status: u16,
    message: &str,
    op: PrivateRestOperation,
) -> (VenueRejectClass, RetrySafety) {
    (
        normalize_reject_class(status, message, op.is_write()),
        retry_safety_for(op, status),
    )
}

pub fn sample_redacted_headers() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("authorization".to_string(), "***redacted***".to_string()),
        ("x-api-key".to_string(), "***redacted***".to_string()),
    ])
}
