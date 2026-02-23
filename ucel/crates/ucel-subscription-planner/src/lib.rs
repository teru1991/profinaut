use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
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

/// cartesian product expansion (params only)
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

// --------------------
// v2 template expansion helpers (NEW)
// --------------------

fn split_canonical_symbol(sym: &str) -> Option<(&str, &str)> {
    // canonical: "BASE/QUOTE"
    let mut it = sym.split('/');
    let b = it.next()?;
    let q = it.next()?;
    if b.is_empty() || q.is_empty() {
        return None;
    }
    Some((b, q))
}

/// Extract template vars from "{var}" occurrences.
/// Example: "{pair}_{contractType}@continuousKline_{interval}" -> {"pair","contractType","interval"}
fn extract_template_vars(tpl: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = tpl.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(j) = tpl[i + 1..].find('}') {
                let name = &tpl[i + 1..i + 1 + j];
                if !name.is_empty() {
                    out.insert(name.to_string());
                }
                i = i + 1 + j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Build variable pool used by coverage_v2 expansion.
/// This makes `{pair}`, `{assetSymbol}`, `{underlyingAsset}` deterministic.
fn build_var_pool(exchange_id: &str, symbols: &[String]) -> HashMap<String, Vec<String>> {
    let mut pool: HashMap<String, Vec<String>> = HashMap::new();

    // 1) {symbol}
    pool.insert("symbol".into(), symbols.to_vec());

    // 2) {pair} from canonical BASE/QUOTE => lowercase base+quote (no slash)
    let mut pairs = BTreeSet::<String>::new();
    for s in symbols {
        if let Some((b, q)) = split_canonical_symbol(s) {
            pairs.insert(format!("{}{}", b.to_lowercase(), q.to_lowercase()));
        }
    }
    if !pairs.is_empty() {
        pool.insert("pair".into(), pairs.into_iter().collect());
    }

    // 3) {assetSymbol}: base+quote assets set (deterministic)
    let mut assets = BTreeSet::<String>::new();
    for s in symbols {
        if let Some((b, q)) = split_canonical_symbol(s) {
            assets.insert(b.to_string());
            assets.insert(q.to_string());
        }
    }
    if !assets.is_empty() {
        pool.insert("assetSymbol".into(), assets.into_iter().collect());
    }

    // 4) {underlyingAsset}: options markPrice streams etc
    // Heuristic: split by '-' or '_' and take first token; else take full symbol.
    if exchange_id == "binance-options" {
        let mut under = BTreeSet::<String>::new();
        for s in symbols {
            let token = s.split('-').next().unwrap_or(s);
            let token = token.split('_').next().unwrap_or(token);
            if !token.is_empty() {
                under.insert(token.to_string());
            } else {
                under.insert(s.clone());
            }
        }
        if !under.is_empty() {
            pool.insert("underlyingAsset".into(), under.into_iter().collect());
        }
    }

    pool
}

fn render_template(mut tpl: String, vars: &HashMap<String, String>) -> String {
    for (k, v) in vars {
        let ph = format!("{{{k}}}");
        if tpl.contains(&ph) {
            tpl = tpl.replace(&ph, v);
        }
    }
    tpl
}

/// Expand a family into concrete topics (and concrete params).
/// - params: cartesian expanded (existing expand_params)
/// - template vars: pulled from var_pool for template vars used by topic_template
///
/// Returns list of (topic, merged_params_object)
/// where merged params includes:
/// - family params
/// - any bound template vars that are NOT "symbol" (optional)
/// - "_topic" : rendered topic string (for adapter consumption)
/// - "_w" : weight
fn expand_family_topics(
    exchange_id: &str,
    family: &FamilyV2,
    symbols: &[String],
    var_pool: &HashMap<String, Vec<String>>,
) -> Result<Vec<(Option<String>, serde_json::Value)>, String> {
    let tpl_vars = extract_template_vars(&family.topic_template);

    // SSOT safety: requires_symbol=false must NOT reference {symbol}
    if !family.requires_symbol && tpl_vars.contains("symbol") {
        return Err(format!(
            "family {}: requires_symbol=false but topic_template references {{symbol}}",
            family.id
        ));
    }

    // expanded family params variants
    let variants = expand_params(&family.params);
    let weight = if family.weight == 0 { default_weight(&family.id) } else { family.weight };

    // Determine which template vars we must bind (excluding "symbol" which comes from SubscriptionKey.symbol)
    let mut need_vars: Vec<String> = tpl_vars.iter().cloned().collect();

    // Prepare pools for required vars other than symbol
    // If template references symbol, we bind it via each symbol, not via pool.
    need_vars.retain(|v| v != "symbol");

    let mut pools: Vec<(String, Vec<String>)> = Vec::new();
    for v in &need_vars {
        let Some(list) = var_pool.get(v) else {
            return Err(format!("family {}: missing var pool for {{{}}}", family.id, v));
        };
        if list.is_empty() {
            return Err(format!("family {}: empty var pool for {{{}}}", family.id, v));
        }
        pools.push((v.clone(), list.clone()));
    }

    // Build cartesian product assignments for those vars
    let mut var_assignments: Vec<HashMap<String, String>> = vec![HashMap::new()];
    for (name, vals) in pools {
        let mut next = Vec::new();
        for base in &var_assignments {
            for val in &vals {
                let mut m = base.clone();
                m.insert(name.clone(), val.clone());
                next.push(m);
            }
        }
        var_assignments = next;
    }

    let mut out: Vec<(Option<String>, serde_json::Value)> = Vec::new();

    // Helper to attach _w/_topic into params object
    let mut attach_meta = |mut params: serde_json::Value, topic: String, weight: u32| -> serde_json::Value {
        if let Some(obj) = params.as_object_mut() {
            obj.insert("_w".into(), serde_json::Value::Number(weight.into()));
            obj.insert("_topic".into(), serde_json::Value::String(topic));
        }
        params
    };

    if family.requires_symbol {
        // requires_symbol => expand for each symbol
        for sym in symbols {
            for p in &variants {
                for va in &var_assignments {
                    let mut all: HashMap<String, String> = va.clone();

                    // bind symbol for rendering
                    all.insert("symbol".into(), sym.clone());

                    // allow params values to be referenced in template as {k}
                    if let Some(obj) = p.as_object() {
                        for (k, v) in obj {
                            let val = if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() };
                            all.insert(k.clone(), val);
                        }
                    }

                    let topic = render_template(family.topic_template.clone(), &all);
                    let params = attach_meta(p.clone(), topic, weight);

                    out.push((Some(sym.clone()), params));
                }
            }
        }
    } else {
        // symbol-less
        for p in &variants {
            for va in &var_assignments {
                let mut all: HashMap<String, String> = va.clone();

                // allow params values to be referenced in template as {k}
                if let Some(obj) = p.as_object() {
                    for (k, v) in obj {
                        let val = if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() };
                        all.insert(k.clone(), val);
                    }
                }

                let topic = render_template(family.topic_template.clone(), &all);
                let params = attach_meta(p.clone(), topic, weight);

                out.push((None, params));
            }
        }
    }

    // Determinism: sort by stable representation of (symbol, canon_params)
    out.sort_by_key(|(sym, params)| {
        let sk = format!(
            "{}|{}|{}|{}",
            exchange_id,
            family.id,
            sym.clone().unwrap_or_default(),
            canon_params(params)
        );
        sk
    });

    Ok(out)
}

/// v2 planner: symbols × families × params × template-vars, then shard by rules.
/// This version also fixes topic rendering into params["_topic"] for adapter usage.
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

    let var_pool = build_var_pool(exchange_id, symbols);

    let mut seed: Vec<SubscriptionKey> = Vec::new();

    for fam in &cov.families {
        let expanded = match expand_family_topics(exchange_id, fam, symbols, &var_pool) {
            Ok(v) => v,
            Err(e) => {
                // Strict coverage means we fail-fast; else degrade with skip (but here we just fail)
                if cov.strict {
                    panic!("coverage_v2 expansion error: {e}");
                } else {
                    continue;
                }
            }
        };

        for (sym_opt, params) in expanded {
            seed.push(SubscriptionKey {
                exchange_id: exchange_id.to_string(),
                op_id: fam.id.clone(),
                symbol: sym_opt,
                params,
            });
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