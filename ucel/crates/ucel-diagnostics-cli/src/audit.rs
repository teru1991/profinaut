use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub ts_rfc3339: String,
    pub actor: String,
    pub action: String,
    pub bundle_id: Option<String>,
    pub diag_semver: Option<String>,
    pub result: String,
    pub reason_code: String,
    pub break_glass: Option<BreakGlassSummary>,
    pub prev_hash_hex: String,
    pub this_hash_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BreakGlassSummary {
    pub ttl_seconds: u64,
    pub reason: String,
    pub approvals_count: usize,
}

pub fn append_audit_event(path: &str, mut ev: AuditEvent) -> Result<(), std::io::Error> {
    ev.prev_hash_hex = read_last_hash(path).unwrap_or_else(|| "0".repeat(64));
    ev.this_hash_hex = compute_event_hash(&ev);

    let line = serde_json::to_string(&ev).expect("audit event json");
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

fn read_last_hash(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let last = content.lines().last()?;
    let v: serde_json::Value = serde_json::from_str(last).ok()?;
    v.get("this_hash_hex")?.as_str().map(ToString::to_string)
}

fn compute_event_hash(ev: &AuditEvent) -> String {
    let stable = serde_json::json!({
        "event_id": ev.event_id,
        "ts_rfc3339": ev.ts_rfc3339,
        "actor": ev.actor,
        "action": ev.action,
        "bundle_id": ev.bundle_id,
        "diag_semver": ev.diag_semver,
        "result": ev.result,
        "reason_code": ev.reason_code,
        "break_glass": ev.break_glass,
        "prev_hash_hex": ev.prev_hash_hex,
    });
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(&stable).expect("stable audit json"));
    hex::encode(h.finalize())
}
