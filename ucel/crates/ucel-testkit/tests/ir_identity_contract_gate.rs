use ucel_core::{IrIssuerIdentityKind, IrIssuerKey, IrMarket};
use ucel_ir::{
    ensure_provenance, normalize_identity_value, IrIssuerIdentityProvenance,
    IrIssuerResolutionInput,
};

#[test]
fn ir_identity_contract_gate() {
    let input = IrIssuerResolutionInput {
        market: IrMarket::Us,
        source_id: "sec_edgar_submissions_api".into(),
        identity_kind: IrIssuerIdentityKind::TickerLike,
        value: "aapl".into(),
    };
    assert_eq!(normalize_identity_value(&input.value), "AAPL");

    let key = IrIssuerKey {
        market: IrMarket::Us,
        canonical_id: "CIK:0000320193".into(),
    };
    assert!(key.canonical_id.starts_with("CIK:"));

    let bad = IrIssuerIdentityProvenance {
        source_id: "".into(),
        evidence: "".into(),
    };
    assert!(ensure_provenance(&bad).is_err());
}
