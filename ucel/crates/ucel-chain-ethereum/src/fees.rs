use crate::errors::{reason_to_error, EvmReasonCode};
use ucel_core::{EvmFeeEstimate, UcelError};

#[derive(Debug, Clone, Copy)]
pub struct FeePolicy {
    pub fee_ceiling: u128,
    pub spike_guard_bps: u32,
}

impl Default for FeePolicy {
    fn default() -> Self {
        Self {
            fee_ceiling: 500_000_000_000,
            spike_guard_bps: 5000,
        }
    }
}

pub fn estimate_eip1559(
    base_fee: u128,
    tip: u128,
    gas_limit: u64,
    policy: FeePolicy,
) -> Result<EvmFeeEstimate, UcelError> {
    let max_fee = base_fee.saturating_mul(2).saturating_add(tip);
    if max_fee > policy.fee_ceiling {
        return Err(reason_to_error(
            EvmReasonCode::ExecutionReverted,
            "fee ceiling exceeded",
        ));
    }
    Ok(EvmFeeEstimate {
        legacy_gas_price: None,
        max_fee_per_gas: Some(max_fee),
        max_priority_fee_per_gas: Some(tip),
        gas_limit,
    })
}

pub fn estimate_legacy(
    gas_price: u128,
    gas_limit: u64,
    policy: FeePolicy,
) -> Result<EvmFeeEstimate, UcelError> {
    if gas_price > policy.fee_ceiling {
        return Err(reason_to_error(
            EvmReasonCode::ExecutionReverted,
            "gas_price exceeds ceiling",
        ));
    }
    Ok(EvmFeeEstimate {
        legacy_gas_price: Some(gas_price),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        gas_limit,
    })
}
