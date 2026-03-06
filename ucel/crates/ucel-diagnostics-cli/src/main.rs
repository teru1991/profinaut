mod args;
mod audit;
mod cmd_analyze;
mod cmd_verify;
mod export;
mod rbac;

use args::{Cli, Command};
use clap::Parser;
use time::OffsetDateTime;

fn main() {
    let cli = Cli::parse();
    let audit_path =
        std::env::var("UCEL_DIAG_AUDIT_PATH").unwrap_or_else(|_| "diagnostics_audit.jsonl".into());

    match cli.cmd {
        Command::Whoami {} => println!("{}", rbac::current_actor()),
        Command::Analyze { input, output } => {
            if let Err(e) = cmd_analyze::run(&input, &output) {
                eprintln!("ANALYZE_FAILED:{e}");
                std::process::exit(1);
            }
        }
        Command::Verify { input } => {
            if let Err(e) = cmd_verify::run(&input) {
                eprintln!("VERIFY_FAILED:{e}");
                std::process::exit(1);
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

            let result = (|| -> Result<audit::BreakGlassSummary, String> {
                let bg = rbac::check_break_glass(ttl_minutes, &reason, &approval)
                    .map_err(|e| format!("RBAC:{e}"))?;
                let bytes = std::fs::read(&input).map_err(|e| e.to_string())?;
                let encrypted = export::encrypt_bundle(&bytes, &recipient_pubkey).map_err(|e| e.to_string())?;
                std::fs::write(&output, encrypted).map_err(|e| e.to_string())?;
                Ok(audit::BreakGlassSummary {
                    ttl_seconds: bg.ttl.as_secs(),
                    reason: bg.reason,
                    approvals_count: bg.approvals.len(),
                })
            })();

            match result {
                Ok(bg_summary) => {
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "export".into(),
                        bundle_id: None,
                        diag_semver: None,
                        result: "ok".into(),
                        reason_code: "OK".into(),
                        break_glass: Some(bg_summary),
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                }
                Err(e) => {
                    let event = audit::AuditEvent {
                        event_id,
                        ts_rfc3339: now,
                        actor,
                        action: "export".into(),
                        bundle_id: None,
                        diag_semver: None,
                        result: "error".into(),
                        reason_code: format!("EXPORT_FAILED:{e}"),
                        break_glass: None,
                        prev_hash_hex: String::new(),
                        this_hash_hex: String::new(),
                    };
                    let _ = audit::append_audit_event(&audit_path, event);
                    eprintln!("EXPORT_FAILED:{e}");
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
