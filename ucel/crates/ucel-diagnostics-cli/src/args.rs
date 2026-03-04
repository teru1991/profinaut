use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ucel-diag")]
#[command(about = "UCEL Diagnostics CLI (Y domain): triage/support bundle/analyze/export with audit + RBAC", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Offline triage: takes a support bundle archive, verifies integrity, analyzes it, outputs summary.json.
    Analyze {
        /// Path to tar.zst support bundle archive
        #[arg(long)]
        input: String,
        /// Output summary.json path
        #[arg(long)]
        output: String,
    },

    /// Export a support bundle for external sharing (encrypted + audited).
    Export {
        /// Path to tar.zst support bundle archive
        #[arg(long)]
        input: String,
        /// Recipient public key file (reference only in this minimum implementation)
        #[arg(long)]
        recipient_pubkey: String,
        /// Output encrypted file path
        #[arg(long)]
        output: String,
        /// Break-glass TTL minutes (required for export)
        #[arg(long)]
        ttl_minutes: u32,
        /// Break-glass reason (required for export)
        #[arg(long)]
        reason: String,
        /// Approval token(s) (can be repeated). Minimal enforcement checks count>=2 unless configured.
        #[arg(long)]
        approval: Vec<String>,
    },

    /// Print current RBAC state (read-only).
    Whoami {},
}
