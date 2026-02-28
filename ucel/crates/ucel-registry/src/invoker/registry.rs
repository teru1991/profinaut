use super::coverage;
use super::{InvokerError, OperationId, VenueId};
use crate::{deribit, CatalogAuth, CatalogEntry, ExchangeCatalog};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
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
                InvokerError::RegistryValidation(format!("missing ws_url for {}", self.spec.id))
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
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let coverage_root = repo_root.join("ucel/coverage");
        let manifests = coverage::discover(&coverage_root)?;
        let mut specs = BTreeMap::new();
        let mut by_venue: BTreeMap<VenueId, BTreeSet<OperationId>> = BTreeMap::new();
        let fixtures = repo_root.join("ucel/fixtures/symbols/strict.json");
        let fixture_raw = std::fs::read_to_string(&fixtures).unwrap_or_default();

        for (venue, manifest) in manifests {
            if manifest.strict && !fixture_raw.contains(&format!("\"{venue}\"")) {
                return Err(InvokerError::RegistryValidation(format!(
                    "strict venue {venue} missing symbol fixture",
                )));
            }
            let catalog = load_catalog_loose(&repo_root, venue.as_str())?;
            let mut catalog_map: BTreeMap<String, (OperationKind, CatalogEntry)> = BTreeMap::new();
            for item in catalog.rest_endpoints {
                catalog_map.insert(item.id.clone(), (OperationKind::Rest, item));
            }
            for item in catalog.ws_channels {
                catalog_map.insert(item.id.clone(), (OperationKind::Ws, item));
            }

            for entry in manifest.entries {
                let id: OperationId = entry.id.parse()?;
                let listed = by_venue.entry(venue.clone()).or_default();
                listed.insert(id.clone());
                if !entry.implemented && !manifest.strict {
                    continue;
                }
                let (kind, spec) = if let Some(found) = catalog_map.get(id.as_str()).cloned() {
                    found
                } else {
                    synthetic_spec(&venue, &id)?
                };
                kind_gate(id.as_str(), kind, &venue)?;
                let key = (venue.clone(), id.clone());
                if specs.insert(key, ResolvedSpec { kind, spec }).is_some() {
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

fn kind_gate(id: &str, kind: OperationKind, venue: &VenueId) -> Result<(), InvokerError> {
    let restish = id.contains(".rest.");
    let wsish = id.contains(".ws.");
    match (restish, wsish, kind) {
        (true, false, OperationKind::Ws) | (false, true, OperationKind::Rest) => Err(
            InvokerError::RegistryValidation(format!("kind gate mismatch for {venue}:{id}")),
        ),
        _ => Ok(()),
    }
}

fn load_catalog_loose(repo_root: &Path, venue: &str) -> Result<ExchangeCatalog, InvokerError> {
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join(venue)
        .join("catalog.json");
    if venue == "deribit" {
        return deribit::load_deribit_catalog_from_path(&path)
            .map_err(|e| InvokerError::RegistryValidation(e.to_string()));
    }
    let raw = std::fs::read_to_string(&path)?;
    serde_json::from_str(&raw).map_err(InvokerError::from)
}

fn synthetic_spec(
    venue: &VenueId,
    id: &OperationId,
) -> Result<(OperationKind, CatalogEntry), InvokerError> {
    let kind = if id.as_str().contains(".ws.")
        || id.as_str().contains(".rpc.")
        || id.as_str().contains("realtime")
    {
        OperationKind::Ws
    } else {
        OperationKind::Rest
    };
    let spec = match kind {
        OperationKind::Rest => CatalogEntry {
            id: id.to_string(),
            visibility: "public".into(),
            requires_auth: Some(false),
            channel: None,
            operation: Some("synthetic-from-coverage".into()),
            method: Some("GET".into()),
            base_url: Some(format!("docs://{venue}")),
            path: Some(format!("/{}", id.as_str().replace('.', "/"))),
            ws_url: None,
            ws: None,
            auth: CatalogAuth::default(),
        },
        OperationKind::Ws => CatalogEntry {
            id: id.to_string(),
            visibility: "public".into(),
            requires_auth: Some(false),
            channel: Some(id.to_string()),
            operation: Some("synthetic-from-coverage".into()),
            method: None,
            base_url: None,
            path: None,
            ws_url: Some(format!("wss://{venue}.invalid/ws")),
            ws: None,
            auth: CatalogAuth::default(),
        },
    };
    Ok((kind, spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_coverage_registry_builds() {
        let reg = SpecRegistry::global().unwrap();
        assert!(!reg.list_venues().is_empty());
    }
}
