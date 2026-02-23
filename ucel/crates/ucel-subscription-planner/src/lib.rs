use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use ucel_ws_rules::ExchangeWsRules;

// --------------------
// Legacy coverage v1
// --------------------

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

pub fn extract_ws_ops(manifest: &CoverageManifest) -> Vec<String> {
    manifest
        .entries
        .iter()
        .filter(|e| e.id.starts_with("crypto.public.ws.") || e.id.starts_with("crypto.private.ws."))
        .map(|e| e.id.clone())
        .collect()
}

pub fn load_manifest(path: &Path) -> Result<CoverageManifest, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionKey {
    pub exchange_id: String,
    pub op_id: String, // op_id or family_id
    pub symbol: Option<String>,
    pub params: serde_json::Value,
}

pub fn stable_key(k: &SubscriptionKey) -> String {
    format!(
        "{}|{}|{}|{}",
        k.exchange_id,
        k.op_id,
        k.symbol.clone().unwrap_or_default(),
        canon_params(&k.params),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnPlan {
    pub conn_id: String,
    pub keys: Vec<String>, // stable key
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub conn_plans: Vec<ConnPlan>,
    pub seed: Vec<SubscriptionKey>,
}

pub fn generate_plan(
    exchange_id: &str,
    ws_ops: &[String],
    symbols: &[String],
    rules: &ExchangeWsRules,
) -> Plan {
    let mut seed = Vec::new();

    let mut prioritized_ops = ws_ops.to_vec();
    prioritized_ops.sort_by_key(|op| {
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

    let mut prioritized_symbols = symbols.to_vec();
    prioritized_symbols.sort_by_key(|s| {
        if s.contains("BTC") {
            0
        } else if s.contains("ETH") {
            1
        } else {
            2
        }
    });

    for op in prioritized_ops {
        for symbol in &prioritized_symbols {
            seed.push(SubscriptionKey {
                exchange_id: exchange_id.to_string(),
                op_id: op.clone(),
                symbol: Some(symbol.clone()),
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

// --------------------
// Coverage v2 (template + params expansion)
// --------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageV2 {
    pub venue: String,
    pub strict: bool,
    pub families: Vec<FamilyV2>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FamilyV2 {
    pub id: String,
    pub requires_symbol: bool,
    pub topic_template: String,
    #[serde(default)]
    pub params: BTreeMap<String, Vec<serde_json::Value>>,
    #[serde(default)]
    pub weight: u32,
}

pub fn load_coverage_v2(path: &Path) -> Result<CoverageV2, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// cartesian product expansion
fn expand_params(params: &BTreeMap<String, Vec<serde_json::Value>>) -> Vec<serde_json::Value> {
    let mut acc: Vec<BTreeMap<String, serde_json::Value>> = vec![BTreeMap::new()];
    for (k, vs) in params {
        let mut next = Vec::new();
        for base in &acc {
            for v in vs {
                let mut m = base.clone();
                m.insert(k.clone(), v.clone());
                next.push(m);
            }
        }
        acc = next;
    }
    acc.into_iter()
        .map(|m| {
            let obj: serde_json::Map<String, serde_json::Value> = m.into_iter().collect();
            serde_json::Value::Object(obj)
        })
        .collect()
}

fn default_weight(family_id: &str) -> u32 {
    let id = family_id.to_lowercase();
    if id.contains("ticker") {
        10
    } else if id.contains("trade") {
        20
    } else if id.contains("book")
        || id.contains("orderbook")
        || id.contains("depth")
        || id.contains("mbp")
    {
        40
    } else if id.contains("kline") || id.contains("candle") || id.contains("ohlc") {
        60
    } else if id.contains("funding")
        || id.contains("openinterest")
        || id.contains("mark")
        || id.contains("index")
    {
        80
    } else {
        90
    }
}

/// v2 planner: symbols × families × params, then shard by rules
pub fn generate_plan_v2(
    exchange_id: &str,
    cov: &CoverageV2,
    symbols: &[String],
    rules: &ExchangeWsRules,
) -> Plan {
    let max_streams = rules.effective_max_streams_per_conn().max(1);
    let max_symbols = rules
        .safety_profile
        .as_ref()
        .and_then(|p| p.max_symbols_per_conn)
        .or(rules.max_symbols_per_conn)
        .unwrap_or(max_streams);

    let mut seed: Vec<SubscriptionKey> = Vec::new();

    for fam in &cov.families {
        let variants = expand_params(&fam.params);
        let weight = if fam.weight == 0 {
            default_weight(&fam.id)
        } else {
            fam.weight
        };

        if fam.requires_symbol {
            for sym in symbols {
                for p in &variants {
                    let mut params = p.clone();
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert("_w".into(), serde_json::Value::Number(weight.into()));
                    }
                    seed.push(SubscriptionKey {
                        exchange_id: exchange_id.to_string(),
                        op_id: fam.id.clone(),
                        symbol: Some(sym.clone()),
                        params,
                    });
                }
            }
        } else {
            for p in &variants {
                let mut params = p.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("_w".into(), serde_json::Value::Number(weight.into()));
                }
                seed.push(SubscriptionKey {
                    exchange_id: exchange_id.to_string(),
                    op_id: fam.id.clone(),
                    symbol: None,
                    params,
                });
            }
        }
    }

    // sort by weight then stable key for determinism
    seed.sort_by_key(|k| {
        let w = k.params.get("_w").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
        (w, stable_key(k))
    });

    // shard (streams + unique symbols)
    let mut conn_plans: Vec<ConnPlan> = Vec::new();
    let mut cur_keys: Vec<String> = Vec::new();
    let mut cur_streams: usize = 0;
    let mut cur_symbols: HashMap<String, ()> = HashMap::new();
    let mut idx = 1usize;

    for k in &seed {
        let sk = stable_key(k);

        let mut next_symbols = cur_symbols.len();
        if let Some(sym) = &k.symbol {
            if !cur_symbols.contains_key(sym) {
                next_symbols += 1;
            }
        }
        let next_streams = cur_streams + 1;

        if next_streams > max_streams || next_symbols > max_symbols {
            conn_plans.push(ConnPlan {
                conn_id: format!("{exchange_id}-conn-{idx}"),
                keys: cur_keys,
                limit: max_streams,
            });
            idx += 1;
            cur_keys = Vec::new();
            cur_streams = 0;
            cur_symbols = HashMap::new();
        }

        cur_streams += 1;
        if let Some(sym) = &k.symbol {
            cur_symbols.insert(sym.clone(), ());
        }
        cur_keys.push(sk);
    }

    if !cur_keys.is_empty() {
        conn_plans.push(ConnPlan {
            conn_id: format!("{exchange_id}-conn-{idx}"),
            keys: cur_keys,
            limit: max_streams,
        });
    }

    Plan { conn_plans, seed }
}
