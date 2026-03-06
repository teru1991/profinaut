use ucel_core::EvmFinalityState;

#[derive(Debug, Clone, Copy)]
pub struct FinalityPolicy {
    pub safe_confirmations: u64,
    pub finalized_confirmations: u64,
}

impl Default for FinalityPolicy {
    fn default() -> Self {
        Self { safe_confirmations: 3, finalized_confirmations: 12 }
    }
}

pub fn finality_from_confirmations(confirmations: u64, policy: FinalityPolicy) -> EvmFinalityState {
    if confirmations >= policy.finalized_confirmations {
        EvmFinalityState::Finalized
    } else if confirmations >= policy.safe_confirmations {
        EvmFinalityState::Safe
    } else if confirmations == 0 {
        EvmFinalityState::Pending
    } else {
        EvmFinalityState::Unsafe
    }
}
