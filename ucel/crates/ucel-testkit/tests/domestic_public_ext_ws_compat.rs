use std::cmp::Ordering;

use ucel_core::{
    validate_vendor_public_ws_runtime_modes, VendorPublicWsReadinessMode, VendorPublicWsResumeMode,
    VendorPublicWsSchemaVersion,
};

#[test]
fn schema_compare_orders_versions() {
    let base = VendorPublicWsSchemaVersion::new(1, 0, 0);
    assert_eq!(
        base.compare(VendorPublicWsSchemaVersion::new(1, 1, 0)),
        Ordering::Less
    );
    assert_eq!(
        base.compare(VendorPublicWsSchemaVersion::new(1, 0, 1)),
        Ordering::Less
    );
    assert_eq!(
        base.compare(VendorPublicWsSchemaVersion::new(1, 0, 0)),
        Ordering::Equal
    );
}

#[test]
fn invalid_runtime_mode_combinations_fail() {
    let err = validate_vendor_public_ws_runtime_modes(
        VendorPublicWsReadinessMode::ImmediateActive,
        ucel_core::VendorPublicWsIntegrityMode::None,
        VendorPublicWsResumeMode::Deadletter,
    );
    assert!(err.is_err());
}

#[test]
fn additive_schema_minor_upgrade_is_supported_pattern() {
    let v1 = VendorPublicWsSchemaVersion::new(1, 0, 0);
    let v2 = VendorPublicWsSchemaVersion::new(1, 1, 0);
    assert!(v2.compare(v1) == Ordering::Greater);
}
