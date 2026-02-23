use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_subscription_planner as planner;
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::WsVenueAdapter;
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::load_for_exchange;

use crate::config::IngestConfig;

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
        ShutdownToken::new(self.flag.clone())
    }

    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

fn now_unix_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64
}

pub async fn run_supervisor(
    cfg: &IngestConfig,
    shutdown: SupervisorShutdown,
) -> Result<Vec<String>, String> {
    let mut started = Vec::new();

    let target_exchanges = determine_exchanges(cfg, &cfg.coverage_dir);
    for exchange_id in target_exchanges {
        if exchange_id != "gmocoin" {
            info!(exchange_id=%exchange_id, "exchange skipped (v1 supports gmocoin only)");
            continue;
        }

        start_exchange(exchange_id.clone(), cfg, shutdown.clone()).await?;
        started.push(exchange_id);
    }

    Ok(started)
}

fn determine_exchanges(cfg: &IngestConfig, coverage_dir: &Path) -> Vec<String> {
    if let Some(allowlist) = &cfg.exchange_allowlist {
        return allowlist.clone();
    }

    let mut from_coverage = Vec::new();
    if let Ok(entries) = std::fs::read_dir(coverage_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    from_coverage.push(stem.to_string());
                }
            }
        }
    }

    if from_coverage.is_empty() {
        vec!["gmocoin".to_string()]
    } else {
        from_coverage
    }
}

async fn start_exchange(
    exchange_id: String,
    cfg: &IngestConfig,
    shutdown: SupervisorShutdown,
) -> Result<(), String> {
    let now = now_unix_i64();

    let rules = load_for_exchange(Path::new(&cfg.rules_dir), &exchange_id);
    let manifest = planner::load_manifest(&cfg.coverage_dir.join(format!("{exchange_id}.yaml")))?;

    let mut ops = planner::extract_ws_ops(&manifest);
    if !cfg.enable_private_ws {
        ops.retain(|id| id.starts_with("crypto.public.ws."));
    }

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(GmoCoinWsAdapter::new());
    let symbols = adapter.fetch_symbols().await?;
    info!(exchange_id=%exchange_id, symbols=%symbols.len(), ops=%ops.len(), "symbols + ops");

    let plan = planner::generate_plan(&exchange_id, &ops, &symbols, &rules);
    if plan.conn_plans.len() > cfg.max_connections_per_exchange {
        return Err(format!(
            "exchange {} requires {} connections over safety guard {}",
            exchange_id,
            plan.conn_plans.len(),
            cfg.max_connections_per_exchange
        ));
    }

    let mut store = SubscriptionStore::open(Path::new(&cfg.store_path))?;
    let mut seed_map = std::collections::HashMap::new();
    for k in &plan.seed {
        seed_map.insert(planner::stable_key(k), k.clone());
    }

    let mut rows = Vec::new();
    for cp in &plan.conn_plans {
        for key in &cp.keys {
            let k = seed_map
                .get(key)
                .ok_or_else(|| format!("seed missing for key={key}"))?;
            rows.push(SubscriptionRow {
                key: key.clone(),
                exchange_id: exchange_id.clone(),
                op_id: k.op_id.clone(),
                symbol: k.symbol.clone(),
                params_canon: planner::canon_params(&k.params),
                assigned_conn: cp.conn_id.clone(),
            });
        }
    }
    store.seed(&rows, now)?;

    let wal = ucel_journal::WalWriter::open(
        Path::new(&cfg.journal_dir),
        cfg.wal_max_bytes,
        cfg.fsync_mode,
    )?;
    let wal = Arc::new(Mutex::new(wal));

    for cp in plan.conn_plans {
        let adapter = adapter.clone();
        let rules = rules.clone();
        let wal = wal.clone();
        let store_path = cfg.store_path.clone();
        let token = shutdown.token();

        let run_cfg = WsRunConfig {
            exchange_id: exchange_id.clone(),
            conn_id: cp.conn_id,
            recv_queue_cap: cfg.recv_queue_cap,
            max_frame_bytes: cfg.max_frame_bytes,
            max_inflight_per_conn: cfg.max_inflight_per_conn,
            connect_timeout: cfg.connect_timeout,
            idle_timeout: cfg.idle_timeout,
            send_queue_hard_limit: cfg.recv_queue_cap,
        };

        tokio::spawn(async move {
            let mut store = match SubscriptionStore::open(Path::new(&store_path)) {
                Ok(s) => s,
                Err(e) => {
                    error!(conn=%run_cfg.conn_id, err=%e, "store open failed");
                    return;
                }
            };

            if let Err(e) =
                run_ws_connection(adapter, rules, &mut store, wal, run_cfg.clone(), token).await
            {
                warn!(conn=%run_cfg.conn_id, err=%e, "connection ended");
            }
        });
    }

    while !shutdown.is_triggered() {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    Ok(())
}
