use crate::obs::{ObsRequiredKeys, ObsSnapshot};
use crate::ws::connection::WsRunConfig;

pub fn transport_bundle(
    ctx: &ObsRequiredKeys,
    snapshot: &ObsSnapshot,
    limits: &WsRunConfig,
    state: &str,
    reconnect_attempts: u32,
) -> serde_json::Value {
    serde_json::json!({
        "required_keys": {
            "exchange_id": ctx.exchange_id,
            "conn_id": ctx.conn_id,
            "op": ctx.op,
            "symbol": ctx.symbol,
            "run_id": ctx.run_id,
            "trace_id": ctx.trace_id,
            "request_id": ctx.request_id,
        },
        "state": state,
        "reconnect_attempts": reconnect_attempts,
        "limits": {
            "out_queue_cap": limits.out_queue_cap,
            "wal_queue_cap": limits.wal_queue_cap,
            "max_frame_bytes": limits.max_frame_bytes,
            "max_inflight_per_conn": limits.max_inflight_per_conn,
            "rl_max_attempts": limits.rl_max_attempts,
        },
        "metrics": snapshot,
    })
}
