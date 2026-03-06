use crate::errors::{reason_to_error, EvmReasonCode};
use ucel_core::{EvmSignedTransaction, EvmTransactionRequest, UcelError};

pub trait EvmSigner: Send + Sync {
    fn sign_transaction(
        &self,
        tx: &EvmTransactionRequest,
    ) -> Result<EvmSignedTransaction, UcelError>;
}

#[derive(Debug, Clone)]
pub struct DeterministicTestSigner {
    pub signer_id: String,
}

impl EvmSigner for DeterministicTestSigner {
    fn sign_transaction(
        &self,
        tx: &EvmTransactionRequest,
    ) -> Result<EvmSignedTransaction, UcelError> {
        if self.signer_id.is_empty() {
            return Err(reason_to_error(
                EvmReasonCode::ExecutionReverted,
                "signer_id empty",
            ));
        }
        let body = format!(
            "{}:{}:{}:{}",
            tx.chain_id.0, tx.from.0, tx.nonce, tx.data_hex
        );
        let hash = format!("0x{:x}", seahash::hash(body.as_bytes()));
        Ok(EvmSignedTransaction {
            tx_hash: hash.clone(),
            raw_tx_hex: format!("0xraw{}", hash.trim_start_matches("0x")),
        })
    }
}

pub fn redact_signer_material(input: &str) -> String {
    input
        .replace("private", "[redacted]")
        .replace("secret", "[redacted]")
}
