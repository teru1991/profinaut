use crate::config::IngestConfig;
use ucel_subscription_planner::{generate_plan, load_all_ws_ops};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_ws_rules::load_for_exchange;

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

        let symbols = vec!["BTC/USDT".to_string()];
        let rules = load_for_exchange(std::path::Path::new(&cfg.rules_dir), &exchange);
        let plan = generate_plan(&exchange, &ws_ops, &symbols, &rules);

        let mut store = SubscriptionStore::open(&cfg.store_path)?;
        let rows: Vec<SubscriptionRow> = plan
            .seed
            .iter()
            .map(|k| SubscriptionRow {
                key: format!(
                    "{}:{}:{}",
                    k.exchange_id,
                    k.op_id,
                    k.symbol.clone().unwrap_or_default()
                ),
                exchange_id: k.exchange_id.clone(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_json: k.params.to_string(),
                assigned_conn: Some(format!("{}-conn-1", exchange)),
            })
            .collect();
        store.seed(&rows, 0)?;
        started.push(exchange);
    }

    Ok(started)
}
