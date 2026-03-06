use super::{InvokerError, OperationId, VenueId};
use crate::hub::registry::exchange_registrations;
use crate::{CatalogEntry, ExchangeCatalog};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Rest,
    Ws,
}

#[derive(Debug, Clone)]
pub struct ResolvedSpec {
    pub kind: OperationKind,
    pub spec: CatalogEntry,
}

impl ResolvedSpec {
    pub fn ws_url(&self) -> Result<String, InvokerError> {
        self.spec
            .ws_url
            .clone()
            .or_else(|| self.spec.ws.as_ref().map(|w| w.url.clone()))
            .ok_or_else(|| {
                InvokerError::RegistryValidation(format!("missing ws url for {}", self.spec.id))
            })
    }
}

#[derive(Debug)]
pub struct SpecRegistry {
    specs: BTreeMap<(VenueId, OperationId), ResolvedSpec>,
    by_venue: BTreeMap<VenueId, BTreeSet<OperationId>>,
}

impl SpecRegistry {
    pub fn global() -> Result<&'static Self, InvokerError> {
        static REG: OnceLock<Result<SpecRegistry, InvokerError>> = OnceLock::new();
        REG.get_or_init(Self::build)
            .as_ref()
            .map_err(|e| InvokerError::RegistryValidation(e.to_string()))
    }

    fn build() -> Result<Self, InvokerError> {
        let mut specs = BTreeMap::new();
        let mut by_venue: BTreeMap<VenueId, BTreeSet<OperationId>> = BTreeMap::new();

        for registration in exchange_registrations() {
            let venue: VenueId = registration.canonical_name.parse()?;
            let catalog = load_catalog_from_registration(registration.catalog_json)?;

            let listed = by_venue.entry(venue.clone()).or_default();
            for item in catalog.rest_endpoints {
                let id: OperationId = item.id.parse()?;
                listed.insert(id.clone());
                let key = (venue.clone(), id.clone());
                if specs
                    .insert(
                        key,
                        ResolvedSpec {
                            kind: OperationKind::Rest,
                            spec: item,
                        },
                    )
                    .is_some()
                {
                    return Err(InvokerError::RegistryValidation(format!(
                        "duplicate id mapping: {venue}:{id}",
                    )));
                }
            }

            for item in catalog.ws_channels {
                let id: OperationId = item.id.parse()?;
                listed.insert(id.clone());
                let key = (venue.clone(), id.clone());
                if specs
                    .insert(
                        key,
                        ResolvedSpec {
                            kind: OperationKind::Ws,
                            spec: item,
                        },
                    )
                    .is_some()
                {
                    return Err(InvokerError::RegistryValidation(format!(
                        "duplicate id mapping: {venue}:{id}",
                    )));
                }
            }
        }

        Ok(Self { specs, by_venue })
    }

    pub fn list_venues(&self) -> Vec<VenueId> {
        self.by_venue.keys().cloned().collect()
    }

    pub fn list_ids(&self, venue: &VenueId) -> Result<Vec<OperationId>, InvokerError> {
        self.by_venue
            .get(venue)
            .map(|x| x.iter().cloned().collect())
            .ok_or_else(|| InvokerError::UnknownVenue(venue.to_string()))
    }

    pub fn list_ws_channels(&self, venue: &VenueId) -> Result<Vec<OperationId>, InvokerError> {
        let ids = self.list_ids(venue)?;
        Ok(ids
            .into_iter()
            .filter(|id| {
                self.specs
                    .get(&(venue.clone(), id.clone()))
                    .map(|s| s.kind == OperationKind::Ws)
                    .unwrap_or(false)
            })
            .collect())
    }

    pub fn resolve(
        &self,
        venue: &VenueId,
        id: &OperationId,
    ) -> Result<&ResolvedSpec, InvokerError> {
        self.specs
            .get(&(venue.clone(), id.clone()))
            .ok_or_else(|| InvokerError::UnknownOperation {
                venue: venue.to_string(),
                id: id.to_string(),
            })
    }
}

fn load_catalog_from_registration(raw: &str) -> Result<ExchangeCatalog, InvokerError> {
    serde_json::from_str(raw).map_err(InvokerError::from)
}

#[allow(dead_code)]
fn _load_catalog_loose(_repo_root: &Path, _venue: &str) -> Result<ExchangeCatalog, InvokerError> {
    Err(InvokerError::RegistryValidation(
        "legacy catalog loader disabled in hub-registry-v1".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_coverage_registry_builds() {
        let reg = SpecRegistry::global().unwrap();
        assert!(reg.list_venues().len() >= 10);
    }

    #[test]
    fn list_ws_channels_is_stable() {
        let reg = SpecRegistry::global().unwrap();
        let venue: VenueId = "binance".parse().unwrap();
        let ws = reg.list_ws_channels(&venue).unwrap();
        assert!(!ws.is_empty());
    }
}
