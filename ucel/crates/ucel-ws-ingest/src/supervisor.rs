use crate::config::IngestConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use ucel_registry::ingest::{find_default_driver, ExchangeIngestDriver, IngestConfigRef, IngestPlanRef, IngestRulesRef, IngestRuntimeRef};
use ucel_subscription_planner::{extract_ws_ops, generate_plan, load_manifest, SubscriptionKey};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_ws_rules::load_for_exchange;

#[derive(Debug, Clone)]
pub struct ExchangeRunReport {
    pub exchange_id: String,
    pub seed_size: usize,
    pub deadletters: usize,
}

pub async fn run_supervisor(cfg: &IngestConfig) -> Result<Vec<ExchangeRunReport>, String> {
    let mut exchanges = vec![
        "binance".to_string(),
        "bybit".to_string(),
        "okx".to_string(),
        "sbivc".to_string(),
    ];
    if let Some(allow) = &cfg.exchange_allowlist {
        exchanges.retain(|x| allow.contains(x));
    }

    let mut store = SubscriptionStore::open(&cfg.store_path)?;
    let coverage_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../coverage");
    let rules_dir = PathBuf::from(&cfg.rules_dir);
    let now = now_ts();
    let mut reports = Vec::new();

    for exchange_id in exchanges {
        let manifest = load_manifest(&coverage_dir.join(format!("{exchange_id}.yaml")))?;
        let ws_ops = extract_ws_ops(&manifest);
        let driver = match find_default_driver(&exchange_id) {
            Some(d) => d,
            None => continue,
        };
        let symbols = match driver.fetch_symbols().await {
            Ok(v) if !v.is_empty() => v,
            Ok(_) => vec!["BTC/USDT".to_string()],
            Err(_e) => {
                reports.push(ExchangeRunReport {
                    exchange_id,
                    seed_size: 0,
                    deadletters: ws_ops.len(),
                });
                continue;
            }
        };

        let rules = load_for_exchange(&rules_dir, &exchange_id);
        let plan = generate_plan(&exchange_id, &ws_ops, &symbols, &rules);
        let assignments = assign_conn_map(&plan.seed, &plan.conn_plans_keys());
        let rows = plan
            .seed
            .iter()
            .map(|k| SubscriptionRow {
                key: key_of(k),
                exchange_id: k.exchange_id.clone(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_json: k.params.to_string(),
                assigned_conn: assignments.get(&key_of(k)).cloned(),
            })
            .collect::<Vec<_>>();

        store.seed(&rows, now)?;

        let runtime = IngestRuntimeRef {
            store_path: cfg.store_path.clone(),
            journal_dir: cfg.journal_dir.clone(),
        };
        let rules_ref = IngestRulesRef {
            support_level: format!("{:?}", rules.support_level),
        };
        let cfg_ref = IngestConfigRef {
            enable_private_ws: cfg.enable_private_ws,
        };
        driver
            .run_ws_ingest(
                IngestPlanRef {
                    exchange_id: exchange_id.clone(),
                    seed_len: plan.seed.len(),
                },
                runtime,
                rules_ref,
                cfg_ref,
            )
            .await?;

        reports.push(ExchangeRunReport {
            exchange_id,
            seed_size: plan.seed.len(),
            deadletters: 0,
        });
    }

    Ok(reports)
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn key_of(k: &SubscriptionKey) -> String {
    format!(
        "{}:{}:{}",
        k.exchange_id,
        k.op_id,
        k.symbol.clone().unwrap_or_default()
    )
}

trait PlanConnKeys {
    fn conn_plans_keys(&self) -> Vec<(String, Vec<String>)>;
}

impl PlanConnKeys for ucel_subscription_planner::Plan {
    fn conn_plans_keys(&self) -> Vec<(String, Vec<String>)> {
        self.conn_plans
            .iter()
            .map(|c| (c.conn_id.clone(), c.keys.clone()))
            .collect()
    }
}

fn assign_conn_map(seed: &[SubscriptionKey], conn_keys: &[(String, Vec<String>)]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (conn_id, keys) in conn_keys {
        for k in keys {
            out.insert(k.clone(), conn_id.clone());
        }
    }
    for k in seed {
        out.entry(key_of(k)).or_insert_with(|| "default-conn".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn allowlist_runs_single_exchange() {
        let cfg = IngestConfig {
            exchange_allowlist: Some(vec!["binance".into()]),
            store_path: ":memory:".into(),
            ..Default::default()
        };
        let v = run_supervisor(&cfg).await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].exchange_id, "binance");
    }
}
