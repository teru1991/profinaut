use ucel_core::{
    IrAccessPattern, IrAccessPolicyClass, IrDocumentFamily, IrIssuerIdentityKind, IrMarket,
    IrSourceDescriptor, IrSourceFamily, IrSourceKind,
};

pub fn build_source_descriptor(
    source_id: &str,
    market: IrMarket,
    source_family: IrSourceFamily,
    source_kind: IrSourceKind,
    access_policy_class: IrAccessPolicyClass,
    access_patterns: Vec<IrAccessPattern>,
) -> IrSourceDescriptor {
    IrSourceDescriptor {
        source_id: source_id.to_string(),
        market,
        source_family,
        source_kind,
        access_policy_class,
        access_patterns,
    }
}

pub fn inventory_taxonomy_supported(
    _identity: IrIssuerIdentityKind,
    _doc: IrDocumentFamily,
) -> bool {
    true
}
