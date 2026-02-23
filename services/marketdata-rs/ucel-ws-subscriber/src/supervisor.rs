use crate::config::IngestConfig;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Mutex;
use tracing::{error, warn};

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_subscription_planner::{canon_params, generate_plan, load_all_ws_ops, stable_key};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::WsVenueAdapter;
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::{load_for_exchange, SupportLevel};

fn should_include_op(op: &str, enable_private_ws: bool) -> bool {
    op.starts_with("crypto.public.ws.")
        || (enable_private_ws && op.starts_with("crypto.private.ws."))
}

#[derive(Clone)]
pub struct SupervisorShutdown {
    pub(crate) flag: Arc<AtomicBool>,
}
impl SupervisorShutdown {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }
    pub fn token(&self) -> ShutdownToken {
        ShutdownToken {
            flag: self.flag.clone(),
        }
    }
}

pub async fn run_supervisor(
    cfg: &IngestConfig,
    shutdown: SupervisorShutdown,
) -> Result<Vec<String>, String> {
    let coverage = load_all_ws_ops(&cfg.coverage_dir)?;
    let mut started = Vec::new();

    let wal = ucel_journal::WalWriter::open(&cfg.journal_dir, cfg.wal_max_bytes, cfg.fsync_mode)
        .map_err(|e| e.to_string())?;
    let wal = Arc::new(Mutex::new(wal));

    for (exchange, ws_ops) in coverage {
        if let Some(allow) = &cfg.exchange_allowlist {
            if !allow.contains(&exchange) {
                continue;
            }
        }
        if exchange == "sbivc" {
            continue;
        }

        if exchange != "gmocoin" {
            continue;
        }

        let rules = load_for_exchange(std::path::Path::new(&cfg.rules_dir), &exchange);
        if matches!(rules.support_level, SupportLevel::NotSupported) {
            continue;
        }

        let ws_ops: Vec<String> = ws_ops
            .into_iter()
            .filter(|op| should_include_op(op, cfg.enable_private_ws))
            .filter(|op| op.starts_with("crypto.public.ws."))
            .collect();
        if ws_ops.is_empty() {
            continue;
        }

        let adapter: Arc<dyn WsVenueAdapter> = Arc::new(GmoCoinWsAdapter::new());
        let symbols = adapter.fetch_symbols().await?;

        let plan = generate_plan(&exchange, &ws_ops, &symbols, &rules);

        if plan.conn_plans.len() > cfg.max_connections_per_exchange {
            return Err(format!(
                "too many connections planned: exchange={exchange} conns={} max={}",
                plan.conn_plans.len(),
                cfg.max_connections_per_exchange
            ));
        }

        let conn_by_key: HashMap<String, String> = plan
            .conn_plans
            .iter()
            .flat_map(|cp| cp.keys.iter().map(|k| (k.clone(), cp.conn_id.clone())))
            .collect();

        {
            let mut store = SubscriptionStore::open(&cfg.store_path)?;
            let rows: Vec<SubscriptionRow> = plan
                .seed
                .iter()
                .map(|k| {
                    let k_stable = stable_key(k);
                    SubscriptionRow {
                        key: k_stable.clone(),
                        exchange_id: k.exchange_id.clone(),
                        op_id: k.op_id.clone(),
                        symbol: k.symbol.clone(),
                        params_json: canon_params(&k.params),
                        assigned_conn: conn_by_key.get(&k_stable).cloned(),
                    }
                })
                .collect();
            store.seed(&rows, now_unix())?;
        }

        for cp in plan.conn_plans.clone() {
            let adapter = adapter.clone();
            let rules = rules.clone();
            let wal = wal.clone();
            let store_path = cfg.store_path.clone();
            let token = shutdown.token();

            let run_cfg = WsRunConfig {
                exchange_id: exchange.clone(),
                conn_id: cp.conn_id.clone(),
                recv_queue_cap: cfg.recv_queue_cap,
                max_frame_bytes: cfg.max_frame_bytes,
                max_inflight_per_conn: cfg.max_inflight_per_conn,
                connect_timeout: cfg.connect_timeout,
                idle_timeout: cfg.idle_timeout,
                reconnect_storm_window: cfg.reconnect_storm_window,
                reconnect_storm_max: cfg.reconnect_storm_max,
            };

            tokio::spawn(async move {
                let mut store = match SubscriptionStore::open(&store_path) {
                    Ok(s) => s,
                    Err(e) => {
                        error!(conn=%run_cfg.conn_id, err=%e, "store open failed");
                        return;
                    }
                };
                if let Err(e) =
                    run_ws_connection(adapter, rules, &mut store, wal, run_cfg, token).await
                {
                    warn!(conn=%cp.conn_id, err=%e, "connection ended");
                }
            });
        }

        started.push(exchange);
    }

    Ok(started)
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
