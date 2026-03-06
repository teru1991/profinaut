use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use ucel_core::{ErrorCode, UcelError};
use ucel_registry::hub::registry;
use ucel_ws_rules::validation::{default_rules_dir_from_repo_root, load_rule_index};

use crate::coverage_v2::{load_jp_resident_access, CoverageV2Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyIssueKind {
    MissingCoverageV2Entry,
    MissingWsRule,
    CatalogCoverageMismatch,
    PolicyEntitlementMismatch,
    LegacyCoverageMismatch,
    UnknownCanonicalName,
    DuplicateAlias,
}

#[derive(Debug, Clone)]
pub struct ConsistencyIssue {
    pub kind: ConsistencyIssueKind,
    pub venue: String,
    pub family: Option<String>,
    pub surface: Option<String>,
    pub source_file: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConsistencyReport {
    pub issues: Vec<ConsistencyIssue>,
    pub warnings: Vec<ConsistencyIssue>,
}

#[derive(Debug, Deserialize)]
struct CoverageV2Venue {
    venue: String,
    #[serde(default)]
    families: Vec<CoverageV2Family>,
}

#[derive(Debug, Deserialize)]
struct CoverageV2Family {
    id: String,
}

#[derive(Debug, Deserialize)]
struct LegacyCoverage {
    venue: String,
    #[serde(default)]
    scope: String,
}

const FAMILY_SPLIT_BRIDGES: &[&str] = &["binance", "bybit", "okx", "bitget", "gmocoin", "htx"];
const EXPLICIT_EXCEPTIONS: &[(&str, &str)] = &[
    ("MissingCoverageV2Entry", "bitbank"),
    ("MissingCoverageV2Entry", "bitflyer"),
    ("MissingCoverageV2Entry", "bithumb"),
    ("MissingCoverageV2Entry", "bitmex"),
    ("MissingCoverageV2Entry", "coinbase"),
    ("MissingCoverageV2Entry", "coincheck"),
    ("MissingCoverageV2Entry", "deribit"),
    ("MissingCoverageV2Entry", "sbivc"),
    ("MissingCoverageV2Entry", "upbit"),
    ("MissingWsRule", "bitget-coin-futures"),
    ("MissingWsRule", "bitget-usdc-futures"),
];

pub fn load_consistency_inputs(repo_root: &Path) -> Result<ConsistencyInputs, UcelError> {
    let coverage_v2_files = list_yaml_files(&repo_root.join("ucel/coverage_v2"))?;
    let coverage_files = list_yaml_files(&repo_root.join("ucel/coverage"))?;
    let ws_rules = load_rule_index(&default_rules_dir_from_repo_root(repo_root))
        .map_err(|e| UcelError::new(ErrorCode::Internal, e))?;
    let policy = load_jp_resident_access(repo_root).map_err(map_cov_error)?;

    Ok(ConsistencyInputs {
        coverage_v2_files,
        coverage_files,
        ws_rules,
        policy,
    })
}

pub struct ConsistencyInputs {
    coverage_v2_files: Vec<PathBuf>,
    coverage_files: Vec<PathBuf>,
    ws_rules: Vec<ucel_ws_rules::validation::WsRuleSummary>,
    policy: crate::coverage_v2::JpResidentAccessPolicy,
}

pub fn build_consistency_report(repo_root: &Path) -> Result<ConsistencyReport, UcelError> {
    let inputs = load_consistency_inputs(repo_root)?;
    let mut report = ConsistencyReport::default();

    let registrations = registry::exchange_registrations();
    let reg_names: BTreeSet<String> = registrations
        .iter()
        .map(|r| r.canonical_name.to_string())
        .collect();

    let mut aliases = BTreeMap::<String, String>::new();
    for reg in registrations {
        for alias in reg.aliases {
            if let Some(prev) =
                aliases.insert(alias.to_ascii_lowercase(), reg.canonical_name.into())
            {
                push_or_except(
                    &mut report,
                    ConsistencyIssue {
                        kind: ConsistencyIssueKind::DuplicateAlias,
                        venue: reg.canonical_name.into(),
                        family: None,
                        surface: None,
                        source_file: None,
                        message: format!(
                            "duplicate alias={} previous={} current={}",
                            alias, prev, reg.canonical_name
                        ),
                    },
                );
            }
        }
    }

    let mut cov2_map: BTreeMap<String, CoverageV2Venue> = BTreeMap::new();
    for f in &inputs.coverage_v2_files {
        let raw = std::fs::read_to_string(f).map_err(io_to_ucel)?;
        let parsed: CoverageV2Venue = serde_yaml::from_str(&raw).map_err(|e| {
            UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("parse {}: {e}", f.display()),
            )
        })?;
        cov2_map.insert(parsed.venue.to_ascii_lowercase(), parsed);
    }

    for canonical in &reg_names {
        let exact = cov2_map.contains_key(canonical);
        let bridged = FAMILY_SPLIT_BRIDGES.contains(&canonical.as_str())
            && cov2_map
                .keys()
                .any(|k| k.starts_with(&format!("{}-", canonical)));
        if !exact && !bridged {
            push_or_except(
                &mut report,
                ConsistencyIssue {
                    kind: ConsistencyIssueKind::MissingCoverageV2Entry,
                    venue: canonical.clone(),
                    family: None,
                    surface: None,
                    source_file: None,
                    message: "registered exchange/family missing in coverage_v2".into(),
                },
            );
        }
    }

    let ws_supported_venues: BTreeSet<String> = cov2_map
        .iter()
        .filter_map(|(venue, data)| {
            let has_ws = data.families.iter().any(|f| f.id.contains(".ws."));
            if has_ws {
                Some(venue.clone())
            } else {
                None
            }
        })
        .collect();

    let ws_rule_ids: BTreeSet<String> = inputs
        .ws_rules
        .iter()
        .map(|r| r.exchange_id.to_ascii_lowercase())
        .collect();

    for venue in &ws_supported_venues {
        if !ws_rule_ids.contains(venue) {
            push_or_except(
                &mut report,
                ConsistencyIssue {
                    kind: ConsistencyIssueKind::MissingWsRule,
                    venue: venue.clone(),
                    family: None,
                    surface: Some("ws".into()),
                    source_file: None,
                    message: "supported ws family missing ws_rules file".into(),
                },
            );
        }
    }

    for rule in &inputs.ws_rules {
        let name = rule.exchange_id.to_ascii_lowercase();
        let known_cov = cov2_map.contains_key(&name);
        let known_reg = reg_names.contains(&name);
        let bridged = FAMILY_SPLIT_BRIDGES
            .iter()
            .any(|base| name.starts_with(&format!("{}-", base)) || *base == name);
        if !known_cov && !known_reg && !bridged {
            push_or_except(
                &mut report,
                ConsistencyIssue {
                    kind: ConsistencyIssueKind::UnknownCanonicalName,
                    venue: name,
                    family: None,
                    surface: Some("ws".into()),
                    source_file: Some(rule.source_file.display().to_string()),
                    message: "ws_rules exchange_id is unknown to coverage_v2/registry".into(),
                },
            );
        }
    }

    for rule in &inputs.ws_rules {
        let venue = rule.exchange_id.to_ascii_lowercase();
        let scope = crate::coverage_v2::jp_scope_for_venue(&inputs.policy, &venue);
        if scope == "public_only" {
            if let Some(ent) = rule.entitlement.as_deref() {
                let e = ent.to_ascii_lowercase();
                if e.contains("private") || e == "public_private" {
                    push_or_except(
                        &mut report,
                        ConsistencyIssue {
                            kind: ConsistencyIssueKind::PolicyEntitlementMismatch,
                            venue: venue.clone(),
                            family: None,
                            surface: Some("ws".into()),
                            source_file: Some(rule.source_file.display().to_string()),
                            message: format!("policy scope={} but ws entitlement={}", scope, ent),
                        },
                    );
                }
            }
        }
    }

    for f in &inputs.coverage_files {
        let raw = std::fs::read_to_string(f).map_err(io_to_ucel)?;
        let parsed: LegacyCoverage = serde_yaml::from_str(&raw).map_err(|e| {
            UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("parse {}: {e}", f.display()),
            )
        })?;
        let legacy_scope = parsed.scope.to_ascii_lowercase();
        let policy_scope = crate::coverage_v2::jp_scope_for_venue(&inputs.policy, &parsed.venue);
        if parsed.venue.eq_ignore_ascii_case("sbivc") && legacy_scope != "public_only" {
            push_or_except(
                &mut report,
                ConsistencyIssue {
                    kind: ConsistencyIssueKind::LegacyCoverageMismatch,
                    venue: parsed.venue.clone(),
                    family: None,
                    surface: None,
                    source_file: Some(f.display().to_string()),
                    message: "sbivc must remain public_only in legacy coverage".into(),
                },
            );
        }
        if ["bitbank", "bitflyer", "coincheck", "gmocoin"].contains(&parsed.venue.as_str())
            && policy_scope == "public_private"
            && legacy_scope == "public_only"
        {
            push_or_except(
                &mut report,
                ConsistencyIssue {
                    kind: ConsistencyIssueKind::LegacyCoverageMismatch,
                    venue: parsed.venue.clone(),
                    family: None,
                    surface: None,
                    source_file: Some(f.display().to_string()),
                    message: "legacy coverage scope contradicts jp policy".into(),
                },
            );
        }
    }

    Ok(report)
}

fn push_or_except(report: &mut ConsistencyReport, issue: ConsistencyIssue) {
    let kind = format!("{:?}", issue.kind);
    if EXPLICIT_EXCEPTIONS
        .iter()
        .any(|(k, v)| *k == kind && issue.venue.eq_ignore_ascii_case(v))
    {
        report.warnings.push(issue);
    } else {
        report.issues.push(issue);
    }
}

pub fn assert_consistent(report: &ConsistencyReport) -> Result<(), UcelError> {
    if report.issues.is_empty() {
        return Ok(());
    }
    let mut lines = Vec::new();
    for issue in &report.issues {
        lines.push(format!(
            "{:?}:{}:{}",
            issue.kind, issue.venue, issue.message
        ));
    }
    Err(UcelError::new(
        ErrorCode::RegistryInvalidCatalog,
        format!("ssot consistency gate failed: {}", lines.join(" | ")),
    ))
}

fn list_yaml_files(dir: &Path) -> Result<Vec<PathBuf>, UcelError> {
    let mut files = Vec::new();
    for ent in std::fs::read_dir(dir).map_err(io_to_ucel)? {
        let p = ent.map_err(io_to_ucel)?.path();
        if p.is_file() && p.extension().map(|e| e == "yaml").unwrap_or(false) {
            files.push(p);
        }
    }
    files.sort();
    Ok(files)
}

fn io_to_ucel(e: std::io::Error) -> UcelError {
    UcelError::new(ErrorCode::Internal, e.to_string())
}

fn map_cov_error(e: CoverageV2Error) -> UcelError {
    UcelError::new(ErrorCode::CatalogInvalid, e.to_string())
}
