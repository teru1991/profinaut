use ucel_core::{ErrorCode, Exchange, OpName, UcelError};

pub struct EthereumAdapter;

impl Exchange for EthereumAdapter {
    fn name(&self) -> &'static str {
        "ethereum"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("{} not implemented for {}", op, self.name()),
        ))
    }
}
