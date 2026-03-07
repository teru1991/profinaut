use crate::domestic_public_inventory::{collect_repo_public_evidence, load_domestic_public_inventory, repo_root, DomesticPublicInventory};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const DOCS_DOMESTIC_VENUES: &[&str] = &["bitbank", "bitflyer", "coincheck", "gmocoin", "bittrade", "sbivc"];

#[derive(Debug, Deserialize)]
pub struct InventoryLock {
    pub version: String,
    pub venues: Vec<String>,
    pub counts: LockCounts,
    pub per_venue: BTreeMap<String, LockVenueCounts>,
    pub stable_identifiers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LockCounts {
    pub total_entries: usize,
    pub rest_entries: usize,
    pub ws_entries: usize,
    pub canonical_core: usize,
    pub canonical_extended: usize,
    pub vendor_public_extension: usize,
    pub not_supported: usize,
}

#[derive(Debug, Deserialize)]
pub struct LockVenueCounts {
    pub total_entries: usize,
    pub rest_entries: usize,
    pub ws_entries: usize,
    pub canonical_core: usize,
    pub canonical_extended: usize,
    pub vendor_public_extension: usize,
    pub not_supported: usize,
}

#[derive(Debug, Default)]
pub struct CompatSummary {
    pub total_entries: usize,
    pub rest_entries: usize,
    pub ws_entries: usize,
    pub canonical_core: usize,
    pub canonical_extended: usize,
    pub vendor_public_extension: usize,
    pub not_supported: usize,
}

pub fn load_inventory_and_lock(root: &Path) -> Result<(DomesticPublicInventory, InventoryLock), Box<dyn std::error::Error>> {
    let inventory = load_domestic_public_inventory(root)?;
    let lock: InventoryLock = serde_json::from_str(&fs::read_to_string(
        root.join("ucel/coverage_v2/domestic_public/jp_public_inventory.lock.json"),
    )?)?;
    Ok((inventory, lock))
}

pub fn collect_workspace_domestic_venues(root: &Path) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut out = BTreeSet::new();
    for entry in fs::read_dir(root.join("ucel/crates"))? {
        let name = entry?.file_name().to_string_lossy().to_string();
        if let Some(v) = name.strip_prefix("ucel-cex-") {
            if DOCS_DOMESTIC_VENUES.contains(&v) {
                out.insert(v.to_string());
            }
        }
    }
    Ok(out)
}

pub fn collect_route_reachability() -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    Ok(collect_repo_public_evidence()?)
}

pub fn collect_docs_matrix_counts(root: &Path) -> Result<BTreeMap<String, usize>, Box<dyn std::error::Error>> {
    let matrix = fs::read_to_string(root.join("ucel/docs/exchanges/domestic_public_compat_matrix.md"))?;
    let mut out = BTreeMap::new();
    for line in matrix.lines() {
        if let Some((k, v)) = line.split_once(":") {
            if k.starts_with("summary.") {
                out.insert(k.trim().to_string(), v.trim().parse::<usize>()?);
            }
        }
    }
    Ok(out)
}

pub fn collect_fixture_coverage(root: &Path) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let files = [
        "ucel/fixtures/domestic_public_goldens/domestic_public_rest_golden.json",
        "ucel/fixtures/domestic_public_goldens/domestic_public_ws_golden.json",
        "ucel/fixtures/domestic_public_goldens/domestic_public_ext_rest_golden.json",
        "ucel/fixtures/domestic_public_goldens/domestic_public_ext_ws_golden.json",
        "ucel/fixtures/domestic_public_goldens/domestic_public_inventory_golden.json",
    ];
    let mut out = BTreeSet::new();
    for f in files {
        let p = root.join(f);
        if p.exists() {
            out.insert(f.to_string());
        }
    }
    Ok(out)
}

pub fn collect_schema_runtime_versions(root: &Path) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let files = [
        "docs/specs/ucel/domestic_public_ext_rest_schema_policy_v1.md",
        "docs/specs/ucel/domestic_public_ext_ws_schema_policy_v1.md",
        "docs/specs/ucel/domestic_public_ws_runtime_v1.md",
        "docs/specs/ucel/domestic_public_ext_ws_runtime_policy_v1.md",
        "docs/specs/ucel/domestic_public_schema_evolution_policy_v1.md",
    ];
    let mut out = BTreeSet::new();
    for f in files {
        if root.join(f).exists() {
            out.insert(f.to_string());
        }
    }
    Ok(out)
}

pub fn summarize_inventory(inventory: &DomesticPublicInventory) -> (CompatSummary, BTreeMap<String, LockVenueCounts>, Vec<String>) {
    let mut summary = CompatSummary::default();
    let mut by_venue: BTreeMap<String, LockVenueCounts> = BTreeMap::new();
    let mut stable = Vec::new();

    for e in &inventory.entries {
        summary.total_entries += 1;
        let venue = by_venue.entry(e.venue.clone()).or_insert(LockVenueCounts {
            total_entries: 0,
            rest_entries: 0,
            ws_entries: 0,
            canonical_core: 0,
            canonical_extended: 0,
            vendor_public_extension: 0,
            not_supported: 0,
        });
        venue.total_entries += 1;

        match e.api_kind.as_str() {
            "rest" => {
                summary.rest_entries += 1;
                venue.rest_entries += 1;
            }
            "ws" => {
                summary.ws_entries += 1;
                venue.ws_entries += 1;
            }
            _ => {}
        }

        match e.surface_class.as_str() {
            "canonical_core" => {
                summary.canonical_core += 1;
                venue.canonical_core += 1;
            }
            "canonical_extended" => {
                summary.canonical_extended += 1;
                venue.canonical_extended += 1;
            }
            "vendor_public_extension" => {
                summary.vendor_public_extension += 1;
                venue.vendor_public_extension += 1;
            }
            "not_supported" => {
                summary.not_supported += 1;
                venue.not_supported += 1;
            }
            _ => {}
        }

        stable.push(format!("{}|{}|{}", e.venue, e.api_kind, e.public_id));
    }

    stable.sort();
    (summary, by_venue, stable)
}

pub fn assert_domestic_public_final_compat(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (inventory, lock) = load_inventory_and_lock(root)?;
    let (summary, by_venue, stable) = summarize_inventory(&inventory);

    if lock.stable_identifiers != stable {
        return Err("inventory stable identifiers mismatch lock".into());
    }
    if lock.venues != inventory.venues {
        return Err("inventory venue list mismatch lock".into());
    }
    if lock.counts.total_entries != summary.total_entries
        || lock.counts.rest_entries != summary.rest_entries
        || lock.counts.ws_entries != summary.ws_entries
        || lock.counts.canonical_core != summary.canonical_core
        || lock.counts.canonical_extended != summary.canonical_extended
        || lock.counts.vendor_public_extension != summary.vendor_public_extension
        || lock.counts.not_supported != summary.not_supported
    {
        return Err("inventory summary mismatch lock".into());
    }
    if lock.per_venue.len() != by_venue.len() {
        return Err("per-venue length mismatch lock".into());
    }
    for (venue, c) in by_venue {
        let lc = lock.per_venue.get(&venue).ok_or("missing lock venue")?;
        if lc.total_entries != c.total_entries
            || lc.rest_entries != c.rest_entries
            || lc.ws_entries != c.ws_entries
            || lc.canonical_core != c.canonical_core
            || lc.canonical_extended != c.canonical_extended
            || lc.vendor_public_extension != c.vendor_public_extension
            || lc.not_supported != c.not_supported
        {
            return Err(format!("per-venue mismatch lock: {venue}").into());
        }
    }

    let repo_routes = collect_route_reachability()?;
    for id in &stable {
        if !repo_routes.contains(id) {
            return Err(format!("route unreachable: {id}").into());
        }
    }

    if inventory.entries.iter().any(|e| e.current_repo_status != "implemented") {
        return Err("partial/not_implemented entry remains".into());
    }

    let workspace_venues = collect_workspace_domestic_venues(root)?;
    let lock_venues: BTreeSet<_> = lock.venues.iter().cloned().collect();
    if workspace_venues != lock_venues {
        return Err("workspace venue drift vs lock".into());
    }

    Ok(())
}

pub fn default_repo_root() -> std::path::PathBuf { repo_root() }
