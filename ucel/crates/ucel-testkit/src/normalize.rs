use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Deterministic JSON normalization for golden assertions.
/// - object keys are sorted recursively
/// - arrays remain ordered by default, but arrays of objects with common
///   market-data keys are stably sorted for deterministic comparison
pub fn canonicalize_json(v: &Value) -> Value {
    match v {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Number(n.clone()),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => {
            let mut out: Vec<Value> = arr.iter().map(canonicalize_json).collect();
            maybe_sort_event_array(&mut out);
            Value::Array(out)
        }
        Value::Object(map) => {
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

fn maybe_sort_event_array(values: &mut [Value]) {
    if values.is_empty() || !values.iter().all(Value::is_object) {
        return;
    }

    // Do not reorder orderbook ladder arrays where sequence matters.
    if values[0].get("price").is_some() && values[0].get("qty").is_some() {
        return;
    }

    values.sort_by(stable_event_cmp);
}

fn stable_event_cmp(a: &Value, b: &Value) -> Ordering {
    compare_key(a, b, "trade_id")
        .then_with(|| compare_key(a, b, "ts"))
        .then_with(|| compare_key(a, b, "price"))
        .then_with(|| compare_key(a, b, "qty"))
}

fn compare_key(a: &Value, b: &Value, key: &str) -> Ordering {
    let av = a.get(key).map(|v| v.to_string()).unwrap_or_default();
    let bv = b.get(key).map(|v| v.to_string()).unwrap_or_default();
    av.cmp(&bv)
}

pub fn first_diff_path(a: &Value, b: &Value) -> Option<String> {
    fn walk(a: &Value, b: &Value, path: &str) -> Option<String> {
        if a == b {
            return None;
        }
        match (a, b) {
            (Value::Object(ma), Value::Object(mb)) => {
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
                        return Some(format!("{path}.{k}"));
                    }
                    if let (Some(va), Some(vb)) = (pa, pb) {
                        if let Some(p) = walk(va, vb, &format!("{path}.{k}")) {
                            return Some(p);
                        }
                    }
                }
                Some(path.to_string())
            }
            (Value::Array(aa), Value::Array(ab)) => {
                let n = aa.len().min(ab.len());
                for i in 0..n {
                    if let Some(p) = walk(&aa[i], &ab[i], &format!("{path}[{i}]")) {
                        return Some(p);
                    }
                }
                Some(format!("{path}[len]"))
            }
            _ => Some(path.to_string()),
        }
    }

    walk(a, b, "$")
}
