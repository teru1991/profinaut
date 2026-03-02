use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GoldenWsFixture {
    pub venue: String,
    pub name: String,
    pub raw: String,
    pub expected: Value,
}

impl GoldenWsFixture {
    pub fn load(repo_root: &Path, venue: &str, name: &str) -> Result<Self, String> {
        // layout:
        // ucel/fixtures/golden/ws/<venue>/
        //   raw.json
        //   expected.normalized.json
        let base = repo_root
            .join("ucel")
            .join("fixtures")
            .join("golden")
            .join("ws")
            .join(venue);

        let raw_path = base.join("raw.json");
        let expected_path = base.join("expected.normalized.json");

        let raw = fs::read_to_string(&raw_path)
            .map_err(|e| format!("failed to read raw fixture {}: {}", raw_path.display(), e))?;

        let expected_raw = fs::read_to_string(&expected_path).map_err(|e| {
            format!(
                "failed to read expected fixture {}: {}",
                expected_path.display(),
                e
            )
        })?;

        let expected: Value = serde_json::from_str(&expected_raw).map_err(|e| {
            format!(
                "failed to parse expected json {}: {}",
                expected_path.display(),
                e
            )
        })?;

        Ok(Self {
            venue: venue.to_string(),
            name: name.to_string(),
            raw,
            expected,
        })
    }
}

/// Canonicalize JSON: recursively sort object keys and canonicalize arrays/values.
/// NOTE: arrays preserve order (intentional). Only object key ordering is normalized.
pub fn canonicalize_json(v: &Value) -> Value {
    match v {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Number(n.clone()),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_json).collect()),
        Value::Object(map) => {
            // stable key ordering
            let mut btm: BTreeMap<String, Value> = BTreeMap::new();
            for (k, vv) in map {
                btm.insert(k.clone(), canonicalize_json(vv));
            }
            let mut out = serde_json::Map::new();
            for (k, vv) in btm {
                out.insert(k, vv);
            }
            Value::Object(out)
        }
    }
}

/// Return the first differing JSON path (best-effort).
pub fn first_diff_path(a: &Value, b: &Value) -> Option<String> {
    fn walk(a: &Value, b: &Value, path: &str) -> Option<String> {
        if a == b {
            return None;
        }
        match (a, b) {
            (Value::Object(ma), Value::Object(mb)) => {
                // keys set differences
                let mut keys: Vec<&String> = ma.keys().collect();
                for k in mb.keys() {
                    if !ma.contains_key(k) {
                        keys.push(k);
                    }
                }
                keys.sort();
                keys.dedup();

                for k in keys {
                    let pa = ma.get(k);
                    let pb = mb.get(k);
                    if pa.is_none() || pb.is_none() {
                        return Some(format!("{}.{k}", path));
                    }
                    if let (Some(va), Some(vb)) = (pa, pb) {
                        if let Some(p) = walk(va, vb, &format!("{}.{k}", path)) {
                            return Some(p);
                        }
                    }
                }
                Some(path.to_string())
            }
            (Value::Array(aa), Value::Array(ab)) => {
                let n = aa.len().min(ab.len());
                for i in 0..n {
                    if let Some(p) = walk(&aa[i], &ab[i], &format!("{}[{}]", path, i)) {
                        return Some(p);
                    }
                }
                Some(format!("{}[len]", path))
            }
            _ => Some(path.to_string()),
        }
    }

    walk(a, b, "$")
}

/// Assert equality with helpful diff output (canonicalized).
pub fn assert_json_eq(actual: &Value, expected: &Value, context: &str) {
    let a = canonicalize_json(actual);
    let e = canonicalize_json(expected);
    if a == e {
        return;
    }

    let diff_path = first_diff_path(&a, &e).unwrap_or_else(|| "$".to_string());
    panic!(
        "golden mismatch ({context})\nfirst_diff_path: {diff_path}\nactual:\n{}\nexpected:\n{}\n",
        serde_json::to_string_pretty(&a).unwrap_or_else(|_| "<json>".into()),
        serde_json::to_string_pretty(&e).unwrap_or_else(|_| "<json>".into()),
    );
}

/// Repo root resolver for tests: ucel/crates/ucel-testkit -> repo root.
pub fn repo_root_from_manifest_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent() // crates
        .and_then(|p| p.parent()) // ucel
        .and_then(|p| p.parent()) // repo root
        .expect("repo root")
        .to_path_buf()
}
