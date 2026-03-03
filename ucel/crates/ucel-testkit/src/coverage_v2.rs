use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum CoverageV2Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("coverage_v2 root not found (expected directory containing coverage_v2 exchange json files)")]
    RootNotFound,
    #[error("missing field: {0}")]
    Missing(&'static str),
    #[error("invalid coverage_v2 file: {path} ({reason})")]
    Invalid { path: String, reason: String },
}

#[derive(Debug, Deserialize)]
pub struct StrictVenues {
    #[serde(default)]
    pub strict_ws_golden: Vec<String>,
    #[serde(default)]
    pub strict_symbol_master: Vec<String>,
}

fn find_dir_named(start: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    if start.file_name().map(|x| x == name).unwrap_or(false) {
        return Some(start.to_path_buf());
    }
    let rd = std::fs::read_dir(start).ok()?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            if let Some(v) = find_dir_named(&p, name, depth - 1) {
                return Some(v);
            }
        }
    }
    None
}

pub fn locate_coverage_v2_root(repo_root: &Path) -> Result<PathBuf, CoverageV2Error> {
    let ucel_cov = repo_root.join("ucel").join("coverage");
    if ucel_cov.join("coverage_v2").is_dir() {
        return Ok(ucel_cov.join("coverage_v2"));
    }
    find_dir_named(&ucel_cov, "coverage_v2", 6).ok_or(CoverageV2Error::RootNotFound)
}

pub fn list_exchange_jsons(coverage_v2_root: &Path) -> Result<Vec<PathBuf>, CoverageV2Error> {
    let candidates = [
        coverage_v2_root.join("exchanges"),
        coverage_v2_root.to_path_buf(),
    ];

    for dir in candidates {
        if dir.is_dir() {
            let mut out = vec![];
            for ent in std::fs::read_dir(&dir)? {
                let p = ent?.path();
                if p.is_file()
                    && p.extension().map(|e| e == "json").unwrap_or(false)
                    && p.file_name().and_then(|n| n.to_str()) != Some("strict_venues.json")
                {
                    out.push(p);
                }
            }
            if !out.is_empty() {
                out.sort();
                return Ok(out);
            }
        }
    }

    let mut out = vec![];
    let mut stack = vec![coverage_v2_root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for ent in std::fs::read_dir(&d)? {
            let p = ent?.path();
            if p.is_dir() {
                stack.push(p.clone());
            }
            if p.is_file()
                && p.extension().map(|e| e == "json").unwrap_or(false)
                && p.file_name().and_then(|n| n.to_str()) != Some("strict_venues.json")
            {
                out.push(p);
            }
        }
    }
    out.sort();
    if out.is_empty() {
        return Err(CoverageV2Error::Invalid {
            path: coverage_v2_root.display().to_string(),
            reason: "no json files found".into(),
        });
    }
    Ok(out)
}

pub fn load_json(path: &Path) -> Result<Value, CoverageV2Error> {
    let bytes = std::fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn load_strict_venues(repo_root: &Path) -> Result<StrictVenues, CoverageV2Error> {
    let root = locate_coverage_v2_root(repo_root)?;
    let path = root.join("strict_venues.json");
    let bytes = std::fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn bool_at(v: &Value, path: &[&str]) -> Option<bool> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_bool()
}

fn str_at<'a>(v: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_str()
}

pub fn infer_exchange_id(path: &Path, v: &Value) -> String {
    if let Some(s) = str_at(v, &["exchange_id"]) {
        return s.to_string();
    }
    if let Some(s) = str_at(v, &["venue"]) {
        return s.to_string();
    }
    if let Some(s) = str_at(v, &["exchange"]) {
        return s.to_string();
    }
    if let Some(s) = str_at(v, &["id"]) {
        return s.to_string();
    }
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn public_rest(v: &Value) -> Result<bool, CoverageV2Error> {
    bool_at(v, &["public", "rest"]).ok_or(CoverageV2Error::Missing("public.rest"))
}

pub fn public_ws(v: &Value) -> Result<bool, CoverageV2Error> {
    bool_at(v, &["public", "ws"]).ok_or(CoverageV2Error::Missing("public.ws"))
}

pub fn private_enabled(v: &Value) -> bool {
    bool_at(v, &["private", "enabled"]).unwrap_or(false)
}
