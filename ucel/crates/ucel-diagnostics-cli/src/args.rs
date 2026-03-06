use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ucel-diag")]
#[command(about = "UCEL Diagnostics CLI (Y domain): support bundle analyze/verify/export", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Analyze a support bundle JSON and emit summary/compat/drift JSON.
    Analyze {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    /// Verify compatibility/drift gates for a support bundle JSON.
    Verify {
        #[arg(long)]
        input: String,
    },
    /// Export an encrypted support bundle archive with audit.
    Export {
        #[arg(long)]
        input: String,
        #[arg(long)]
        recipient_pubkey: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        ttl_minutes: u32,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        approval: Vec<String>,
    },
    Whoami {},
}
