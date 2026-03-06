use crate::loader::load_for_exchange;
use crate::model::SupportLevel;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WsRuleSummary {
    pub exchange_id: String,
    pub source_file: PathBuf,
    pub entitlement: Option<String>,
    pub support_level: SupportLevel,
}

pub fn list_rule_files(rules_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for ent in std::fs::read_dir(rules_dir).map_err(|e| format!("read rules dir failed: {e}"))? {
        let p = ent
            .map_err(|e| format!("read rules entry failed: {e}"))?
            .path();
        if p.is_file() && p.extension().map(|e| e == "toml").unwrap_or(false) {
            files.push(p);
        }
    }
    files.sort();
    Ok(files)
}

pub fn load_rule_index(rules_dir: &Path) -> Result<Vec<WsRuleSummary>, String> {
    let mut out = Vec::new();
    for file in list_rule_files(rules_dir)? {
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid rule file name: {}", file.display()))?;
        let rule = load_for_exchange(rules_dir, stem);
        out.push(WsRuleSummary {
            exchange_id: rule.exchange_id,
            source_file: file,
            entitlement: rule.entitlement,
            support_level: rule.support_level,
        });
    }
    Ok(out)
}

pub fn default_rules_dir_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root
        .join("ucel")
        .join("crates")
        .join("ucel-ws-rules")
        .join("rules")
}
