use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_journal::{FsyncMode, WalWriter};
use ucel_subscription_planner as planner;
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::WsVenueAdapter;
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::load_for_exchange;

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

pub async fn run(
    coverage_dir: &std::path::Path,
    rules_dir: &std::path::Path,
    store_path: &std::path::Path,
    journal_dir: &std::path::Path,
    shutdown: SupervisorShutdown,
) -> Result<(), String> {
    let exchange_id = "gmocoin".to_string();
    let now = now_unix_i64();

    let rules = load_for_exchange(rules_dir, &exchange_id);
    let manifest = planner::load_manifest(&coverage_dir.join(format!("{exchange_id}.yaml")))?;

    let mut ops = planner::extract_ws_ops(&manifest);
    ops.retain(|id| id.starts_with("crypto.public.ws."));

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(GmoCoinWsAdapter::new());
    let symbols = adapter.fetch_symbols().await?;
    info!(exchange_id=%exchange_id, symbols=%symbols.len(), ops=%ops.len(), "symbols + ops");

    let plan = planner::generate_plan(&exchange_id, &ops, &symbols, &rules);

    let mut store = SubscriptionStore::open(store_path)?;
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

    let wal = WalWriter::open(journal_dir, 256 * 1024 * 1024, FsyncMode::Balanced)?;
    let wal = Arc::new(Mutex::new(wal));

    for cp in plan.conn_plans.clone() {
        let adapter = adapter.clone();
        let rules = rules.clone();
        let wal = wal.clone();
        let store_path = store_path.to_path_buf();
        let token = shutdown.token();

        let run_cfg = WsRunConfig {
            exchange_id: exchange_id.clone(),
            conn_id: cp.conn_id.clone(),
            recv_queue_cap: 4096,
            max_frame_bytes: 4 * 1024 * 1024,
            max_inflight_per_conn: 64,
            connect_timeout: std::time::Duration::from_secs(10),
            idle_timeout: std::time::Duration::from_secs(30),
            send_queue_hard_limit: 4096,
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
