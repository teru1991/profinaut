use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GoldenWsCase {
    pub venue: String,
    pub case_name: String,
    pub endpoint_id: String,
    pub raw_payload: String,
    pub expected: Value,
    pub raw_path: PathBuf,
    pub expected_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RawEnvelope {
    endpoint_id: String,
    payload: Value,
}

pub fn repo_root_from_manifest_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

pub fn discover_ws_cases(repo_root: &Path, venue: &str) -> Result<Vec<GoldenWsCase>, String> {
    let venue_dir = repo_root
        .join("ucel")
        .join("fixtures")
        .join("golden")
        .join("ws")
        .join(venue);

    if !venue_dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in
        fs::read_dir(&venue_dir).map_err(|e| format!("read_dir {}: {}", venue_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("read_dir item {}: {e}", venue_dir.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let case_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| format!("invalid case dir under {}", venue_dir.display()))?;
        out.push(load_case(venue, case_name, &path)?);
    }

    out.sort_by(|a, b| a.case_name.cmp(&b.case_name));
    Ok(out)
}

fn load_case(venue: &str, case_name: &str, case_dir: &Path) -> Result<GoldenWsCase, String> {
    let raw_json = case_dir.join("raw.json");
    let raw_txt = case_dir.join("raw.txt");
    let expected_path = case_dir.join("expected.normalized.json");

    let raw_path = if raw_json.exists() {
        raw_json
    } else if raw_txt.exists() {
        raw_txt
    } else {
        return Err(format!(
            "missing raw fixture (raw.json/raw.txt) for venue={venue} case={case_name} dir={}",
            case_dir.display()
        ));
    };

    if !expected_path.exists() {
        return Err(format!(
            "missing expected.normalized.json for venue={venue} case={case_name} dir={}",
            case_dir.display()
        ));
    }

    let raw = fs::read_to_string(&raw_path)
        .map_err(|e| format!("read raw fixture {}: {}", raw_path.display(), e))?;

    let parsed_raw: RawEnvelope = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "parse raw envelope {} (require endpoint_id/payload): {}",
            raw_path.display(),
            e
        )
    })?;

    let expected_raw = fs::read_to_string(&expected_path)
        .map_err(|e| format!("read expected fixture {}: {}", expected_path.display(), e))?;
    let expected: Value = serde_json::from_str(&expected_raw)
        .map_err(|e| format!("parse expected fixture {}: {}", expected_path.display(), e))?;

    Ok(GoldenWsCase {
        venue: venue.to_string(),
        case_name: case_name.to_string(),
        endpoint_id: parsed_raw.endpoint_id,
        raw_payload: parsed_raw.payload.to_string(),
        expected,
        raw_path,
        expected_path,
    })
}
