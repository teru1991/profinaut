use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use ucel_ws_rules::ExchangeWsRules;

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageManifest {
    pub venue: String,
    pub strict: bool,
    pub entries: Vec<CoverageEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageEntry {
    pub id: String,
    pub implemented: bool,
    pub tested: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionKey {
    pub exchange_id: String,
    pub op_id: String,
    pub symbol: Option<String>,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnPlan {
    pub conn_id: String,
    pub keys: Vec<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub conn_plans: Vec<ConnPlan>,
    pub seed: Vec<SubscriptionKey>,
}

pub fn load_manifest(path: &Path) -> Result<CoverageManifest, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

pub fn extract_ws_ops(manifest: &CoverageManifest) -> Vec<String> {
    manifest
        .entries
        .iter()
        .filter(|e| e.id.starts_with("crypto.public.ws.") || e.id.starts_with("crypto.private.ws."))
        .map(|e| e.id.clone())
        .collect()
}

pub fn load_all_ws_ops(coverage_dir: &Path) -> Result<Vec<(String, Vec<String>)>, String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(coverage_dir)
        .map_err(|e| format!("read_dir {}: {e}", coverage_dir.display()))?
    {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("yaml") {
            continue;
        }
        let m = load_manifest(&path)?;
        let fname = path
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        if m.venue != fname {
            return Err(format!("venue mismatch {} != {}", m.venue, fname));
        }
        out.push((m.venue.clone(), extract_ws_ops(&m)));
    }
    Ok(out)
}

pub fn stable_key(k: &SubscriptionKey) -> String {
    format!(
        "{}|{}|{}|{}",
        k.exchange_id,
        k.op_id,
        k.symbol.clone().unwrap_or_default(),
        canon_params(&k.params)
    )
}

pub fn canon_params(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(map) => {
            let mut sorted: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (k, vv) in map {
                if vv.is_null() {
                    continue;
                }
                sorted.insert(k.clone(), vv.clone());
            }
            let obj: serde_json::Map<String, serde_json::Value> = sorted.into_iter().collect();
            serde_json::Value::Object(obj).to_string()
        }
        _ => v.to_string(),
    }
}

pub fn generate_plan(
    exchange_id: &str,
    ws_ops: &[String],
    symbols: &[String],
    rules: &ExchangeWsRules,
) -> Plan {
    let mut seed = Vec::new();
    let mut ops = ws_ops.to_vec();
    ops.sort_by_key(|op| {
        if op.contains("orderbook") {
            0
        } else if op.contains("trade") {
            1
        } else if op.contains("ticker") {
            2
        } else {
            3
        }
    });

    for op in ops {
        for sym in symbols {
            seed.push(SubscriptionKey {
                exchange_id: exchange_id.to_string(),
                op_id: op.clone(),
                symbol: Some(sym.clone()),
                params: json!({}),
            });
        }
    }

    let limit = rules.effective_max_streams_per_conn().max(1);
    let mut conn_plans = Vec::new();
    for (i, chunk) in seed.chunks(limit).enumerate() {
        conn_plans.push(ConnPlan {
            conn_id: format!("{exchange_id}-conn-{}", i + 1),
            keys: chunk.iter().map(stable_key).collect(),
            limit,
        });
    }

    Plan { conn_plans, seed }
}
