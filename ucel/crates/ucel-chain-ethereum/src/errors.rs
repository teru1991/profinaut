use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmReasonCode {
    ProviderTimeout,
    ProviderChainMismatch,
    NonceTooLow,
    NonceTooHigh,
    ReplacementUnderpriced,
    InsufficientFunds,
    ExecutionReverted,
    ReceiptTimeout,
    ReorgDetected,
    UnsupportedRpcMethod,
}

pub fn reason_to_error(reason: EvmReasonCode, message: impl Into<String>) -> UcelError {
    let code = match reason {
        EvmReasonCode::ProviderTimeout => ErrorCode::Timeout,
        EvmReasonCode::ProviderChainMismatch => ErrorCode::CatalogInvalid,
        EvmReasonCode::NonceTooLow | EvmReasonCode::NonceTooHigh => ErrorCode::Desync,
        EvmReasonCode::ReplacementUnderpriced
        | EvmReasonCode::InsufficientFunds
        | EvmReasonCode::ExecutionReverted => ErrorCode::InvalidOrder,
        EvmReasonCode::ReceiptTimeout => ErrorCode::Timeout,
        EvmReasonCode::ReorgDetected => ErrorCode::Desync,
        EvmReasonCode::UnsupportedRpcMethod => ErrorCode::NotSupported,
    };
    UcelError::new(code, message)
}
