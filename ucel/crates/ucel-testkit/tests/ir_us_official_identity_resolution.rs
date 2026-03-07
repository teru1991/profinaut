use ucel_core::{IrIssuerIdentityKind, IrMarket};
use ucel_ir::{sec_adapter, IrIssuerResolutionInput, IrSourceAdapter};

#[test]
fn ir_us_official_identity_resolution() {
    let a = sec_adapter();
    let by_ticker = a
        .resolve_issuer(&IrIssuerResolutionInput {
            market: IrMarket::Us,
            source_id: "sec_edgar_submissions_api".into(),
            identity_kind: IrIssuerIdentityKind::TickerLike,
            value: "AAPL".into(),
        })
        .expect("resolve ticker");
    assert!(by_ticker.issuer_key.canonical_id.contains("0000320193"));
    assert!(!by_ticker.provenance.evidence.is_empty());

    let err = a
        .resolve_issuer(&IrIssuerResolutionInput {
            market: IrMarket::Us,
            source_id: "sec_edgar_submissions_api".into(),
            identity_kind: IrIssuerIdentityKind::TickerLike,
            value: "UNKNOWN".into(),
        })
        .expect_err("unknown should fail");
    assert!(err.to_string().contains("IssuerNotFound") || err.to_string().contains("issuer not found"));
}
