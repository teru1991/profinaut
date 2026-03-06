use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ucel_core::{PrivateRestOperation, PrivateRestSupport, VenueAccessScope, VenueRejectClass};
use ucel_registry::hub::ExchangeId;
use ucel_registry::{default_capabilities_for_exchange, load_catalog_from_repo_root};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateRestVenueMatrix {
    pub venue: String,
    pub support: BTreeMap<PrivateRestOperation, PrivateRestSupport>,
}

pub trait PrivateRestClient {
    fn support_matrix(&self, exchange: ExchangeId) -> PrivateRestVenueMatrix;
}

#[derive(Debug, Default)]
pub struct PrivateRestFacade {
    pub repo_root: String,
}

impl PrivateRestFacade {
    pub fn new(repo_root: impl Into<String>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub fn venue_support_matrix(&self, exchange: ExchangeId) -> PrivateRestVenueMatrix {
        let venue = exchange.as_str().to_string();
        let mut support = BTreeMap::new();
        let scope = default_capabilities_for_exchange(exchange)
            .ok()
            .and_then(|c| c.venue_access.map(|v| v.scope))
            .unwrap_or(VenueAccessScope::PublicOnly);

        if !matches!(scope, VenueAccessScope::PublicPrivate) {
            for op in all_private_rest_operations() {
                support.insert(op, PrivateRestSupport::BlockedByPolicy);
            }
            return PrivateRestVenueMatrix { venue, support };
        }

        let catalog_path = Path::new(&self.repo_root);
        let rest_ids: BTreeSet<String> = load_catalog_from_repo_root(catalog_path, &venue)
            .ok()
            .map(|c| {
                c.rest_endpoints
                    .iter()
                    .map(|e| e.id.to_ascii_lowercase())
                    .collect()
            })
            .unwrap_or_default();

        for op in all_private_rest_operations() {
            let status = match op {
                PrivateRestOperation::GetBalances => has_any(&rest_ids, &["balance", "assets"]),
                PrivateRestOperation::GetOpenOrders => {
                    has_any(&rest_ids, &["openorders", "open_orders"])
                }
                PrivateRestOperation::GetOrder => has_any(&rest_ids, &["order.get", "order.query"]),
                PrivateRestOperation::CancelOrder => has_any(&rest_ids, &["cancel"]),
                PrivateRestOperation::GetFills => {
                    has_any(&rest_ids, &["fills", "matchresults", "executions"])
                }
                PrivateRestOperation::GetAccountProfile => {
                    has_any(&rest_ids, &["account", "profile"])
                }
                PrivateRestOperation::GetPositions => {
                    has_any(&rest_ids, &["position", "positions"])
                }
            };
            support.insert(
                op,
                if status {
                    PrivateRestSupport::Supported
                } else {
                    PrivateRestSupport::Partial
                },
            );
        }

        PrivateRestVenueMatrix { venue, support }
    }
}

impl PrivateRestClient for PrivateRestFacade {
    fn support_matrix(&self, exchange: ExchangeId) -> PrivateRestVenueMatrix {
        self.venue_support_matrix(exchange)
    }
}

fn has_any(ids: &BTreeSet<String>, needles: &[&str]) -> bool {
    ids.iter().any(|id| needles.iter().any(|n| id.contains(n)))
}

pub fn reason_class_from_http(status: u16, message: &str) -> VenueRejectClass {
    ucel_core::normalize_reject_class(status, message, false)
}

pub fn all_private_rest_operations() -> [PrivateRestOperation; 7] {
    [
        PrivateRestOperation::GetBalances,
        PrivateRestOperation::GetOpenOrders,
        PrivateRestOperation::GetOrder,
        PrivateRestOperation::CancelOrder,
        PrivateRestOperation::GetFills,
        PrivateRestOperation::GetAccountProfile,
        PrivateRestOperation::GetPositions,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_scope_maps_to_blocked_matrix() {
        let facade = PrivateRestFacade::new("/tmp/not-used");
        let matrix = facade.venue_support_matrix(ExchangeId::Sbivc);
        assert!(matrix
            .support
            .values()
            .all(|s| matches!(s, PrivateRestSupport::BlockedByPolicy)));
    }
}
