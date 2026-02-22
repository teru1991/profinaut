use crate::config::IngestConfig;
use std::collections::HashMap;
use ucel_subscription_planner::{generate_plan, load_all_ws_ops};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_ws_rules::{load_for_exchange, SupportLevel};

fn should_include_op(op: &str, enable_private_ws: bool) -> bool {
    op.starts_with("crypto.public.ws.") || (enable_private_ws && op.starts_with("crypto.private.ws."))
}

fn subscription_key(exchange_id: &str, op_id: &str, symbol: Option<&str>) -> String {
    format!("{exchange_id}:{op_id}:{}", symbol.unwrap_or_default())
}

pub async fn run_supervisor(cfg: &IngestConfig) -> Result<Vec<String>, String> {
    let coverage_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage");
    let coverage = load_all_ws_ops(&coverage_dir)?;
    let mut started = Vec::new();

    for (exchange, ws_ops) in coverage {
        if let Some(allow) = &cfg.exchange_allowlist {
            if !allow.contains(&exchange) {
                continue;
            }
        }
        if exchange == "sbivc" {
            // unsupported in v1 baseline
            continue;
        }

        let rules = load_for_exchange(std::path::Path::new(&cfg.rules_dir), &exchange);
        if matches!(rules.support_level, SupportLevel::NotSupported) {
            continue;
        }
        let ws_ops: Vec<String> = ws_ops
            .into_iter()
            .filter(|op| should_include_op(op, cfg.enable_private_ws))
            .collect();
        if ws_ops.is_empty() {
            continue;
        }

        let symbols = vec!["BTC/USDT".to_string()];
        let plan = generate_plan(&exchange, &ws_ops, &symbols, &rules);
        let conn_by_key: HashMap<String, String> = plan
            .conn_plans
            .iter()
            .flat_map(|cp| cp.keys.iter().map(|k| (k.clone(), cp.conn_id.clone())))
            .collect();

        let mut store = SubscriptionStore::open(&cfg.store_path)?;
        let rows: Vec<SubscriptionRow> = plan
            .seed
            .iter()
            .map(|k| SubscriptionRow {
                key: subscription_key(&k.exchange_id, &k.op_id, k.symbol.as_deref()),
                exchange_id: k.exchange_id.clone(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_json: k.params.to_string(),
                assigned_conn: conn_by_key
                    .get(&subscription_key(&k.exchange_id, &k.op_id, k.symbol.as_deref()))
                    .cloned()
                    .or_else(|| Some(format!("{}-conn-1", exchange))),
            })
            .collect();
        store.seed(&rows, 0)?;
        started.push(exchange);
    }

    Ok(started)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_private_ops_only_when_enabled() {
        assert!(should_include_op("crypto.public.ws.trades.trade", false));
        assert!(!should_include_op(
            "crypto.private.ws.userdata.executionreport",
            false
        ));
        assert!(should_include_op(
            "crypto.private.ws.userdata.executionreport",
            true
        ));
    }
}
