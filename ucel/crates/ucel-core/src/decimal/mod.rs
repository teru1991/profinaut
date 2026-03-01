//! UCEL Decimal policy SSOT.
//! Centralizes rounding/tick-step/guard rules.
pub mod guard;
pub mod policy;
pub mod serde;
pub mod tick_step;

pub use guard::{DecimalGuard, DecimalGuardError};
pub use policy::{DecimalPolicy, DecimalPolicyError};
pub use tick_step::{
    QuantizeMode, StepSize as CoreStepSize, TickSize as CoreTickSize, TickStepError,
};
