use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use ucel_registry::hub::registry::{list_public_rest_entries, list_public_ws_entries};
use ucel_registry::hub::ExchangeId;

const DOMESTIC_VENUES: &[&str] = &[
    "bitbank",
    "bitflyer",
    "coincheck",
    "gmocoin",
    "bittrade",
    "sbivc",
];

#[derive(Debug, Deserialize)]
pub struct DomesticPublicInventory {
    pub version: String,
    pub venues: Vec<String>,
    pub entries: Vec<DomesticPublicEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomesticPublicEntry {
    pub venue: String,
    pub venue_family: String,
    pub api_kind: String,
    pub public_id: String,
    pub path_or_channel: String,
    pub method_or_subscribe_kind: String,
    pub auth: String,
    pub category: String,
    pub canonical_surface: String,
    pub surface_class: String,
    pub current_repo_status: String,
    pub evidence_files: Vec<String>,
    pub evidence_kinds: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Default)]
pub struct InventoryDiff {
    pub missing_from_inventory: Vec<String>,
    pub unknown_in_inventory: Vec<String>,
    pub duplicate_keys: Vec<String>,
    pub non_public_auth: Vec<String>,
    pub missing_evidence_files: Vec<String>,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn inventory_path(repo_root: &Path) -> PathBuf {
    repo_root.join("ucel/coverage_v2/domestic_public/jp_public_inventory.json")
}

pub fn load_domestic_public_inventory(
    repo_root: &Path,
) -> Result<DomesticPublicInventory, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(inventory_path(repo_root))?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn collect_repo_public_evidence() -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut out = BTreeSet::new();
    for venue in DOMESTIC_VENUES {
        let exchange = match *venue {
            "bitbank" => ExchangeId::Bitbank,
            "bitflyer" => ExchangeId::Bitflyer,
            "coincheck" => ExchangeId::Coincheck,
            "gmocoin" => ExchangeId::Gmocoin,
            "bittrade" => ExchangeId::Bittrade,
            "sbivc" => ExchangeId::Sbivc,
            _ => unreachable!(),
        };
        for entry in list_public_rest_entries(exchange)? {
            out.insert(format!("{venue}|rest|{}", entry.id));
        }
        for entry in list_public_ws_entries(exchange)? {
            out.insert(format!("{venue}|ws|{}", entry.id));
        }
    }
    Ok(out)
}

pub fn compare_inventory_to_repo(repo_root: &Path, inv: &DomesticPublicInventory) -> InventoryDiff {
    let mut diff = InventoryDiff::default();
    let mut keys = BTreeSet::new();
    let mut inv_keys = BTreeSet::new();

    for entry in &inv.entries {
        let key = format!("{}|{}|{}", entry.venue, entry.api_kind, entry.public_id);
        if !keys.insert(key.clone()) {
            diff.duplicate_keys.push(key.clone());
        }
        inv_keys.insert(key.clone());

        if entry.auth != "public" {
            diff.non_public_auth.push(key.clone());
        }
        for file in &entry.evidence_files {
            if !repo_root.join(file).exists() {
                diff.missing_evidence_files
                    .push(format!("{} => {}", key, file));
            }
        }
    }

    if let Ok(repo_evidence) = collect_repo_public_evidence() {
        for key in &repo_evidence {
            if !inv_keys.contains(key) {
                diff.missing_from_inventory.push(key.clone());
            }
        }
        for key in inv_keys {
            if !repo_evidence.contains(&key) {
                diff.unknown_in_inventory.push(key);
            }
        }
    }

    diff
}

pub fn assert_domestic_public_inventory_complete(
    repo_root: &Path,
) -> Result<DomesticPublicInventory, Box<dyn std::error::Error>> {
    let inventory = load_domestic_public_inventory(repo_root)?;
    let diff = compare_inventory_to_repo(repo_root, &inventory);

    if !diff.missing_from_inventory.is_empty()
        || !diff.unknown_in_inventory.is_empty()
        || !diff.duplicate_keys.is_empty()
        || !diff.non_public_auth.is_empty()
        || !diff.missing_evidence_files.is_empty()
    {
        return Err(format!("inventory gate failed: {diff:?}").into());
    }
    Ok(inventory)
}

pub fn summarize_by_venue(
    entries: &[DomesticPublicEntry],
) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut out = BTreeMap::new();
    for e in entries {
        let counters = out.entry(e.venue.clone()).or_insert_with(BTreeMap::new);
        *counters
            .entry(format!("api_kind:{}", e.api_kind))
            .or_insert(0) += 1;
        *counters
            .entry(format!("surface_class:{}", e.surface_class))
            .or_insert(0) += 1;
        if e.current_repo_status == "not_implemented" {
            *counters.entry("status:not_implemented".into()).or_insert(0) += 1;
        }
    }
    out
}
