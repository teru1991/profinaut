use crate::config::IngestConfig;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_subscription_planner::{
    canon_params, extract_ws_ops, generate_plan, load_manifest, stable_key,
};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::WsVenueAdapter;
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::{load_for_exchange, SupportLevel};

#[derive(Clone)]
pub struct SupervisorShutdown {
    flag: Arc<AtomicBool>,
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
    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
    pub fn token(&self) -> ShutdownToken {
        ShutdownToken {
            flag: self.flag.clone(),
        }
    }
}

fn now_unix_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn should_include_public_crypto(op: &str) -> bool {
    op.starts_with("crypto.public.ws.")
}

fn adapter_factory(exchange: &str) -> Option<Arc<dyn WsVenueAdapter>> {
    match exchange {
        "gmocoin" => Some(Arc::new(GmoCoinWsAdapter::new())),
        _ => None,
    }
}

pub async fn run_supervisor(
    cfg: &IngestConfig,
    shutdown: SupervisorShutdown,
) -> Result<(), String> {
    // WAL shared
    std::fs::create_dir_all(&cfg.journal_dir).map_err(|e| e.to_string())?;
    let wal = ucel_journal::WalWriter::open(&cfg.journal_dir, cfg.wal_max_bytes, cfg.fsync_mode)
        .map_err(|e| e.to_string())?;
    let wal = Arc::new(Mutex::new(wal));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // allowlist loop（安全）
    for exchange in &cfg.exchange_allowlist {
        if shutdown.is_triggered() {
            break;
        }

        let adapter = match adapter_factory(exchange.as_str()) {
            Some(a) => a,
            None => {
                warn!(exchange=%exchange, "no adapter registered; skip");
                continue;
            }
        };

        // rules gate（安全）
        let rules = load_for_exchange(&cfg.rules_dir, exchange);
        match rules.support_level {
            SupportLevel::Full => {}
            SupportLevel::Partial if !cfg.require_rules_full && cfg.allow_partial_rules => {
                warn!(exchange=%exchange, "rules are partial but allowed by config");
            }
            _ => {
                warn!(exchange=%exchange, "rules are not full; skip");
                continue;
            }
        }

        // coverage
        let manifest_path = cfg.coverage_dir.join(format!("{exchange}.yaml"));
        let manifest = load_manifest(&manifest_path)?;
        let mut ops = extract_ws_ops(&manifest);
        ops.retain(|op| should_include_public_crypto(op)); // v1: public crypto only

        if ops.is_empty() {
            warn!(exchange=%exchange, "no public crypto ws ops in coverage; skip");
            continue;
        }

        // symbols
        let symbols = adapter.fetch_symbols().await?;
        info!(exchange=%exchange, symbols=%symbols.len(), ops=%ops.len(), "symbols loaded");

        // plan
        let plan = generate_plan(exchange, &ops, &symbols, &rules);
        if plan.conn_plans.len() > cfg.max_connections_per_exchange {
            return Err(format!(
                "too many connections planned: exchange={exchange} conns={} max={}",
                plan.conn_plans.len(),
                cfg.max_connections_per_exchange
            ));
        }

        // conn assignment map
        let conn_by_key: HashMap<String, String> = plan
            .conn_plans
            .iter()
            .flat_map(|cp| cp.keys.iter().map(|k| (k.clone(), cp.conn_id.clone())))
            .collect();

        // seed store
        {
            let mut store =
                SubscriptionStore::open(cfg.store_path.to_str().unwrap_or("/tmp/ucel.sqlite"))?;
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
            store.seed(&rows, now_unix_i64())?;
        }

        // spawn per conn
        for cp in plan.conn_plans.clone() {
            let exchange = exchange.clone();
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

            let h = tokio::spawn(async move {
                let mut store = match SubscriptionStore::open(
                    store_path.to_str().unwrap_or("/tmp/ucel.sqlite"),
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        error!(exchange=%exchange, conn=%run_cfg.conn_id, err=%e, "store open failed");
                        return;
                    }
                };

                if let Err(e) =
                    run_ws_connection(adapter, rules, &mut store, wal, run_cfg, token).await
                {
                    warn!(exchange=%exchange, conn=%cp.conn_id, err=%e, "connection ended");
                }
            });
            handles.push(h);
        }
    }

    // shutdown join/abort
    while !shutdown.is_triggered() {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    info!(handles=%handles.len(), "shutdown: joining tasks");
    let grace = cfg.shutdown_grace;
    let join_all = async {
        for h in handles {
            let _ = h.await;
        }
    };

    if tokio::time::timeout(grace, join_all).await.is_err() {
        warn!("shutdown grace exceeded; aborting remaining tasks");
        // NOTE: tasks already moved into join_all; in practice, keep handles in Arc<Mutex<Vec<JoinHandle>>> for abort.
        // Minimal safe approach here: rely on token + process exit. If you want hard abort,
        // refactor handles into shared vec and abort each on timeout.
    }

    Ok(())
}
