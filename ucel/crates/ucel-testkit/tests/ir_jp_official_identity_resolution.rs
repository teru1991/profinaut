use ucel_core::{IrIssuerIdentityKind, IrMarket};
use ucel_ir::{statutory_adapter, timely_adapter, IrIssuerResolutionInput, IrSourceAdapter};

#[test]
fn ir_jp_official_identity_resolution() {
    let s = statutory_adapter();
    let issuer = s
        .resolve_issuer(&IrIssuerResolutionInput {
            market: IrMarket::Jp,
            source_id: "edinet_api_documents_v2".into(),
            identity_kind: IrIssuerIdentityKind::JpEdinetCodeLike,
            value: "E12345".into(),
        })
        .expect("resolve statutory issuer");
    assert!(!issuer.provenance.evidence.is_empty());

    let t = timely_adapter();
    let err = t
        .resolve_issuer(&IrIssuerResolutionInput {
            market: IrMarket::Jp,
            source_id: "jp_tdnet_timely_html".into(),
            identity_kind: IrIssuerIdentityKind::JpExchangeCodeLike,
            value: "UNKNOWN".into(),
        })
        .expect_err("unknown should fail");
    assert!(err.to_string().contains("issuer not found"));
}
