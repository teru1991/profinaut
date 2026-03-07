use crate::errors::{UcelIrError, UcelIrErrorKind};
use ucel_core::{normalize_alias, IrIssuerAlias, IrIssuerIdentityKind, IrIssuerKey, IrMarket};

#[derive(Debug, Clone)]
pub struct IrIssuerIdentityProvenance {
    pub source_id: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrIssuerConfidence(pub f32);

#[derive(Debug, Clone)]
pub struct IrIssuerResolutionInput {
    pub market: IrMarket,
    pub source_id: String,
    pub identity_kind: IrIssuerIdentityKind,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct IrIssuerResolutionResult {
    pub issuer_key: IrIssuerKey,
    pub aliases: Vec<IrIssuerAlias>,
    pub provenance: IrIssuerIdentityProvenance,
    pub confidence: IrIssuerConfidence,
}

pub trait IrIssuerResolver {
    fn resolve_issuer(
        &self,
        input: &IrIssuerResolutionInput,
    ) -> Result<IrIssuerResolutionResult, UcelIrError>;

    fn resolve_aliases(&self, issuer_key: &IrIssuerKey) -> Result<Vec<IrIssuerAlias>, UcelIrError>;
}

pub fn ensure_provenance(provenance: &IrIssuerIdentityProvenance) -> Result<(), UcelIrError> {
    if provenance.source_id.trim().is_empty() || provenance.evidence.trim().is_empty() {
        return Err(UcelIrError::new(
            UcelIrErrorKind::Policy,
            "identity provenance is required",
        ));
    }
    Ok(())
}

pub fn normalize_identity_value(value: &str) -> String {
    normalize_alias(value)
}
