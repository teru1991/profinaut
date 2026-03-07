use ucel_core::{IrAccessDecision, IrAccessPolicyClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrAccessViolationReason {
    PaidOrContract,
    LoginRequired,
    PolicyBlocked,
    UnknownPolicy,
    AntiBotBypassRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrAccessGuard;

impl IrAccessGuard {
    pub fn evaluate(policy: IrAccessPolicyClass) -> IrAccessDecision {
        policy.to_access_decision()
    }

    pub fn ensure_allowed(
        policy: IrAccessPolicyClass,
    ) -> Result<IrAccessDecision, IrAccessViolationReason> {
        match policy {
            IrAccessPolicyClass::FreePublicNoAuthAllowed => Ok(IrAccessDecision::Allowed),
            IrAccessPolicyClass::FreePublicNoAuthReviewRequired => {
                Ok(IrAccessDecision::ReviewRequired)
            }
            IrAccessPolicyClass::ExcludedPaidOrContract => {
                Err(IrAccessViolationReason::PaidOrContract)
            }
            IrAccessPolicyClass::ExcludedLoginRequired => {
                Err(IrAccessViolationReason::LoginRequired)
            }
            IrAccessPolicyClass::ExcludedPolicyBlocked => {
                Err(IrAccessViolationReason::PolicyBlocked)
            }
        }
    }
}
