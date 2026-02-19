use ucel_core::{ErrorCode, Exchange, OpName, UcelError};

pub struct GmoCoinAdapter;

impl Exchange for GmoCoinAdapter {
    fn name(&self) -> &'static str {
        "gmo_coin"
    }

    fn execute(&self, op: OpName) -> Result<(), UcelError> {
        Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("{} not implemented for {}", op, self.name()),
        ))
    }
}
