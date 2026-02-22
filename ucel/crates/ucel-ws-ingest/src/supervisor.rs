use crate::config::IngestConfig;
use std::path::Path;
use ucel_journal::{FsyncMode, WalWriter};
use ucel_registry::ingest::registered_ingest_driver_ids;
use ucel_subscription_planner::{extract_ws_ops, generate_plan, load_manifest, SubscriptionKey};
use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_ws_rules::load_for_exchange;

pub async fn run_supervisor(cfg: &IngestConfig) -> Result<Vec<String>, String> {
    let mut exchanges: Vec<String> = registered_ingest_driver_ids()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    if let Some(allow) = &cfg.exchange_allowlist {
        exchanges.retain(|x| allow.contains(x));
    }

    for exchange_id in &exchanges {
        initialize_exchange(exchange_id, cfg)?;
    }

    Ok(exchanges)
}

fn initialize_exchange(exchange_id: &str, cfg: &IngestConfig) -> Result<(), String> {
    let coverage_path = Path::new(&cfg.coverage_dir).join(format!("{exchange_id}.yaml"));
    let manifest = load_manifest(&coverage_path)?;

    let ws_ops = extract_ws_ops(&manifest);
    let public_ops: Vec<String> = ws_ops
        .iter()
        .filter(|op| op.starts_with("crypto.public.ws."))
        .cloned()
        .collect();
    let private_ops: Vec<String> = ws_ops
        .iter()
        .filter(|op| op.starts_with("crypto.private.ws."))
        .cloned()
        .collect();

    let rules = load_for_exchange(Path::new(&cfg.rules_dir), exchange_id);
    let plan = generate_plan(exchange_id, &public_ops, &cfg.default_symbols, &rules);

    let mut store = SubscriptionStore::open(&cfg.store_path)?;
    let rows: Vec<SubscriptionRow> = plan.seed.iter().map(|k| to_row(k, exchange_id)).collect();
    store.seed(&rows, now_ts())?;

    let private_plan = generate_plan(exchange_id, &private_ops, &cfg.default_symbols, &rules);
    let private_rows: Vec<SubscriptionRow> = private_plan
        .seed
        .iter()
        .map(|k| to_row(k, exchange_id))
        .collect();
    store.seed(&private_rows, now_ts())?;
    if !cfg.enable_private_ws {
        deadletter_private_ops(&store, exchange_id, &private_ops, &cfg.default_symbols)?;
    }

    let mut wal = WalWriter::open(
        Path::new(&cfg.journal_dir).join(exchange_id),
        1024 * 1024,
        FsyncMode::Balanced,
    )?;
    wal.append(&ucel_journal::RawRecord {
        ts: now_ts() as u64,
        exchange_id: exchange_id.to_string(),
        conn_id: "bootstrap".into(),
        op_id: "ingest.bootstrap".into(),
        symbol: None,
        raw_bytes_b64: "e30=".into(),
        meta: serde_json::json!({"seed": rows.len(), "private_ws_enabled": cfg.enable_private_ws}),
    })?;

    Ok(())
}

fn deadletter_private_ops(
    store: &SubscriptionStore,
    exchange_id: &str,
    private_ops: &[String],
    symbols: &[String],
) -> Result<(), String> {
    let now = now_ts();
    for op in private_ops {
        for symbol in symbols {
            let key = format!("{exchange_id}:{op}:{symbol}");
            store.mark_deadletter(
                &key,
                "private ws credentials missing or disabled",
                now,
            )?;
        }
    }
    Ok(())
}

pub fn should_reconnect_on_stall(last_message_age_secs: u64, stall_threshold_secs: u64) -> bool {
    last_message_age_secs >= stall_threshold_secs
}

fn to_row(k: &SubscriptionKey, exchange_id: &str) -> SubscriptionRow {
    SubscriptionRow {
        key: format!(
            "{}:{}:{}",
            exchange_id,
            k.op_id,
            k.symbol.clone().unwrap_or_default()
        ),
        exchange_id: exchange_id.to_string(),
        op_id: k.op_id.clone(),
        symbol: k.symbol.clone(),
        params_json: k.params.to_string(),
        assigned_conn: Some(format!("{exchange_id}-conn-1")),
    }
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
