use ucel_core::{
    vendor_public_ws_operation_specs, VendorPublicWsIntegrityMode, VendorPublicWsReadinessMode,
    VendorPublicWsResumeMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtPublicRuleView {
    pub readiness_mode: VendorPublicWsReadinessMode,
    pub integrity_mode: VendorPublicWsIntegrityMode,
    pub resume_mode: VendorPublicWsResumeMode,
}

pub fn ext_public_rule_view(venue: &str, operation_id: &str) -> Option<ExtPublicRuleView> {
    vendor_public_ws_operation_specs()
        .iter()
        .find(|x| x.venue == venue && x.operation_id == operation_id)
        .map(|x| ExtPublicRuleView {
            readiness_mode: x.readiness_mode,
            integrity_mode: x.integrity_mode,
            resume_mode: x.resume_mode,
        })
}
