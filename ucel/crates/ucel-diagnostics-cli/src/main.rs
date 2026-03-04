mod args;
mod audit;
mod export;
mod rbac;

use args::{Cli, Command};
use clap::Parser;
use std::fs;
use time::OffsetDateTime;

fn main() {
    let cli = Cli::parse();
    let audit_path =
        std::env::var("UCEL_DIAG_AUDIT_PATH").unwrap_or_else(|_| "diagnostics_audit.jsonl".into());

    match cli.cmd {
        Command::Whoami {} => {
            println!("{}", rbac::current_actor());
        }
        Command::Analyze { input, output } => {
            let actor = rbac::current_actor();
            let now = now_rfc3339();
            let event_id = uuid::Uuid::new_v4().to_string();

            let result = (|| -> Result<(String, String), String> {
                let bytes = fs::read(&input).map_err(|e| e.to_string())?;
                let (manifest, summary) =
                    ucel_diagnostics_analyzer::analyze_tar_zst_bundle(bytes).map_err(|e| e.to_string())?;
                let summary_json = serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?;
                fs::write(&output, summary_json).map_err(|e| e.to_string())?;
                Ok((manifest.bundle_id, manifest.diag_semver))
            })();

            match result {
                Ok((bundle_id, diag_semver)) => {
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "analyze".into(),
                        bundle_id: Some(bundle_id),
                        diag_semver: Some(diag_semver),
                        result: "ok".into(),
                        reason_code: "OK".into(),
                        break_glass: None,
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                }
                Err(e) => {
                    let reason = format!("ANALYZE_FAILED:{e}");
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "analyze".into(),
                        bundle_id: None,
                        diag_semver: None,
                        result: "error".into(),
                        reason_code: reason.clone(),
                        break_glass: None,
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                    eprintln!("{reason}");
                    std::process::exit(1);
                }
            }
        }
        Command::Export {
            input,
            recipient_pubkey,
            output,
            ttl_minutes,
            reason,
            approval,
        } => {
            let actor = rbac::current_actor();
            let now = now_rfc3339();
            let event_id = uuid::Uuid::new_v4().to_string();

            let result = (|| -> Result<(String, String, audit::BreakGlassSummary), String> {
                let bg = rbac::check_break_glass(ttl_minutes, &reason, &approval)
                    .map_err(|e| format!("RBAC:{e}"))?;
                let _ = bg.approved_at;

                let bytes = fs::read(&input).map_err(|e| e.to_string())?;
                let (manifest, _) =
                    ucel_diagnostics_analyzer::analyze_tar_zst_bundle(bytes.clone()).map_err(|e| e.to_string())?;
                let encrypted = export::encrypt_bundle(&bytes, &recipient_pubkey).map_err(|e| e.to_string())?;
                fs::write(&output, encrypted).map_err(|e| e.to_string())?;

                Ok((
                    manifest.bundle_id,
                    manifest.diag_semver,
                    audit::BreakGlassSummary {
                        ttl_seconds: bg.ttl.as_secs(),
                        reason: bg.reason,
                        approvals_count: bg.approvals.len(),
                    },
                ))
            })();

            match result {
                Ok((bundle_id, diag_semver, bg_summary)) => {
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "export".into(),
                        bundle_id: Some(bundle_id),
                        diag_semver: Some(diag_semver),
                        result: "ok".into(),
                        reason_code: "OK".into(),
                        break_glass: Some(bg_summary),
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                }
                Err(e) => {
                    let reason = format!("EXPORT_FAILED:{e}");
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "export".into(),
                        bundle_id: None,
                        diag_semver: None,
                        result: "error".into(),
                        reason_code: reason.clone(),
                        break_glass: None,
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                    eprintln!("{reason}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}
