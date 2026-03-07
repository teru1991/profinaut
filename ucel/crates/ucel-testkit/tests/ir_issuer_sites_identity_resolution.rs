use ucel_core::{IrIssuerIdentityKind, IrMarket};
use ucel_ir::{jp_issuer_html_adapter, IrIssuerResolutionInput, IrSourceAdapter};

#[test]
fn ir_issuer_sites_identity_resolution() {
    let a = jp_issuer_html_adapter();
    let resolved = a
        .resolve_issuer(&IrIssuerResolutionInput {
            market: IrMarket::Jp,
            source_id: "jp_issuer_ir_html_public".into(),
            identity_kind: IrIssuerIdentityKind::IssuerSiteSlugLike,
            value: "acme-jp".into(),
        })
        .expect("resolve by slug");
    assert_eq!(resolved.issuer_key.canonical_id, "JP-ACME-1111");
    assert!(!resolved.provenance.evidence.is_empty());
    assert!(resolved.aliases.iter().any(|a| a.value.contains("https://")));
}
