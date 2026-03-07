use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct IrInventory {
    pub version: String,
    pub markets: Vec<String>,
    pub sources: Vec<IrSource>,
    pub identities: Vec<IrIdentity>,
    pub documents: Vec<IrDocument>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrSource {
    pub market: String,
    pub source_family: String,
    pub source_id: String,
    pub source_kind: String,
    pub access_policy_class: String,
    pub access_patterns: Vec<String>,
    pub issuer_identity_kind: Vec<String>,
    pub document_family: Vec<String>,
    pub artifact_kind: Vec<String>,
    pub current_repo_status: String,
    pub evidence_files: Vec<String>,
    pub evidence_kinds: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrIdentity {
    pub market: String,
    pub identity_kind: String,
    pub source_id: String,
    pub canonical_role: String,
    pub evidence_files: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrDocument {
    pub market: String,
    pub source_id: String,
    pub document_family: String,
    pub artifact_kind: String,
    pub issuer_identity_kind: String,
    pub access_pattern: String,
    pub access_policy_class: String,
    pub current_repo_status: String,
    pub evidence_files: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Default)]
pub struct IrInventoryDiff {
    pub missing_from_inventory: Vec<String>,
    pub missing_evidence_files: Vec<String>,
    pub duplicate_sources: Vec<String>,
    pub source_without_policy: Vec<String>,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn load_ir_inventory(root: &Path) -> Result<IrInventory, Box<dyn std::error::Error>> {
    let path = root.join("ucel/coverage_v2/ir/ir_inventory.json");
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn collect_repo_ir_evidence(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if root.join("ucel/crates/ucel-ir/src/providers/edinet/mod.rs").exists() {
        out.insert("edinet_api_documents_v2".to_string());
    }
    if root
        .join("ucel/crates/ucel-ir/src/providers/sec_edgar/mod.rs")
        .exists()
    {
        out.insert("sec_edgar_submissions_api".to_string());
    }
    let spec = root.join("docs/specs/ucel/ir_connector_spec.md");
    if let Ok(body) = fs::read_to_string(spec) {
        if body.contains("tdnet") {
            out.insert("jp_tdnet_timely_html".to_string());
        }
        if body.contains("web/html/pdf") || body.contains("source_type") {
            out.insert("jp_issuer_ir_html_public".to_string());
            out.insert("jp_issuer_ir_feed_public".to_string());
            out.insert("us_issuer_ir_html_public".to_string());
            out.insert("us_issuer_ir_feed_public".to_string());
        }
    }
    out
}

pub fn compare_ir_inventory_to_repo(root: &Path, inv: &IrInventory) -> IrInventoryDiff {
    let mut diff = IrInventoryDiff::default();
    let evidence = collect_repo_ir_evidence(root);
    let mut source_ids = BTreeSet::new();

    for s in &inv.sources {
        if !source_ids.insert(format!("{}|{}", s.market, s.source_id)) {
            diff.duplicate_sources
                .push(format!("{}|{}", s.market, s.source_id));
        }
        if s.access_policy_class.trim().is_empty() {
            diff.source_without_policy.push(s.source_id.clone());
        }
        for f in &s.evidence_files {
            if !root.join(f).exists() {
                diff.missing_evidence_files
                    .push(format!("source:{} => {}", s.source_id, f));
            }
        }
    }

    let inv_ids: BTreeSet<String> = inv.sources.iter().map(|s| s.source_id.clone()).collect();
    for src in evidence {
        if !inv_ids.contains(&src) {
            diff.missing_from_inventory.push(src);
        }
    }

    diff
}

pub fn assert_ir_inventory_complete(root: &Path) -> Result<IrInventory, Box<dyn std::error::Error>> {
    let inv = load_ir_inventory(root)?;
    let diff = compare_ir_inventory_to_repo(root, &inv);
    if !diff.missing_from_inventory.is_empty()
        || !diff.missing_evidence_files.is_empty()
        || !diff.duplicate_sources.is_empty()
        || !diff.source_without_policy.is_empty()
    {
        return Err(format!("ir inventory gate failed: {diff:?}").into());
    }
    Ok(inv)
}

pub fn assert_ir_access_policy_complete(inv: &IrInventory) -> Result<(), Box<dyn std::error::Error>> {
    let allowed = BTreeSet::from([
        "free_public_noauth_allowed",
        "free_public_noauth_review_required",
        "excluded_paid_or_contract",
        "excluded_login_required",
        "excluded_policy_blocked",
    ]);
    for s in &inv.sources {
        if !allowed.contains(s.access_policy_class.as_str()) {
            return Err(format!("unknown policy class for source {}", s.source_id).into());
        }
        if s.access_policy_class.starts_with("excluded_") && s.current_repo_status == "implemented" {
            return Err(format!("excluded source is implemented: {}", s.source_id).into());
        }
    }
    for d in &inv.documents {
        if !allowed.contains(d.access_policy_class.as_str()) {
            return Err(format!("unknown policy class for document {}", d.source_id).into());
        }
    }
    Ok(())
}

pub fn inventory_counts(inv: &IrInventory) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    out.insert("sources".into(), inv.sources.len());
    out.insert("identities".into(), inv.identities.len());
    out.insert("documents".into(), inv.documents.len());
    out.insert(
        "implemented".into(),
        inv.sources
            .iter()
            .filter(|s| s.current_repo_status == "implemented")
            .count(),
    );
    out.insert(
        "partial".into(),
        inv.sources
            .iter()
            .filter(|s| s.current_repo_status == "partial")
            .count(),
    );
    out.insert(
        "not_implemented".into(),
        inv.sources
            .iter()
            .filter(|s| s.current_repo_status == "not_implemented")
            .count(),
    );
    out
}
