pub mod balance;
pub mod errors;
pub mod fees;
pub mod finality;
pub mod logs;
pub mod nonce;
pub mod provider;
pub mod receipt;
pub mod reorg;
pub mod resume;
pub mod signer;
pub mod tx;
pub mod types;

use ucel_core::{ErrorCode, Exchange, OpName, UcelError};

pub use balance::{get_block_number, get_chain_id, get_erc20_balance, get_native_balance};
pub use errors::{reason_to_error, EvmReasonCode};
pub use fees::{estimate_eip1559, estimate_legacy, FeePolicy};
pub use finality::{finality_from_confirmations, FinalityPolicy};
pub use logs::{cursor_after, get_logs, subscribe_logs};
pub use nonce::NonceManager;
pub use provider::{EvmHttpProvider, EvmProviderPolicy, EvmProviderSet, EvmWsProvider};
pub use receipt::{get_receipt, wait_for_receipt};
pub use reorg::{detect_reorg, replay_range_for_reorg};
pub use resume::{dedup_logs, resume_cursor};
pub use signer::{redact_signer_material, DeterministicTestSigner, EvmSigner};
pub use tx::{build_transaction, send_raw_transaction, sign_transaction};
pub use types::{EvmProviderInfo, ProviderError, ProviderResponse};

pub struct EthereumAdapter;

impl Exchange for EthereumAdapter {
    fn name(&self) -> &'static str {
        "ethereum"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        match op {
            OpName::FetchBalances
            | OpName::FetchStatus
            | OpName::FetchLatestExecutions
            | OpName::FetchOpenPositions => Ok(()),
            _ => Err(UcelError::new(
                ErrorCode::NotSupported,
                format!("{} not implemented for {}", op, self.name()),
            )),
        }
    }
}
