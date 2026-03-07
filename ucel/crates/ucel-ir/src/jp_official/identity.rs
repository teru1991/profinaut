use super::errors::{JpOfficialError, JpOfficialErrorCode};
use crate::identity::{
    IrIssuerConfidence, IrIssuerIdentityProvenance, IrIssuerResolutionInput,
    IrIssuerResolutionResult,
};
use serde::Deserialize;
use ucel_core::{IrIssuerAlias, IrIssuerIdentityKind, IrIssuerKey, IrMarket};

#[derive(Debug, Deserialize)]
struct FixtureAlias {
    kind: String,
    value: String,
    provenance: String,
}

#[derive(Debug, Deserialize)]
struct FixtureIssuer {
    source_id: String,
    canonical_id: String,
    aliases: Vec<FixtureAlias>,
}

fn issuers() -> Result<Vec<FixtureIssuer>, JpOfficialError> {
    serde_json::from_str(include_str!(
        "../../../../../ucel/fixtures/ir_jp_official/issuers.json"
    ))
    .map_err(|e| JpOfficialError::new(JpOfficialErrorCode::ParseFailed, e.to_string()))
}

fn parse_kind(kind: &str) -> Option<IrIssuerIdentityKind> {
    Some(match kind {
        "jp_edinet_code_like" => IrIssuerIdentityKind::JpEdinetCodeLike,
        "jp_local_code_like" => IrIssuerIdentityKind::JpLocalCodeLike,
        "jp_exchange_code_like" => IrIssuerIdentityKind::JpExchangeCodeLike,
        "url_like" => IrIssuerIdentityKind::UrlLike,
        _ => return None,
    })
}

pub fn resolve(
    input: &IrIssuerResolutionInput,
) -> Result<IrIssuerResolutionResult, JpOfficialError> {
    let candidates = issuers()?
        .into_iter()
        .filter(|x| x.source_id == input.source_id)
        .filter(|x| {
            x.aliases
                .iter()
                .any(|a| a.value.eq_ignore_ascii_case(&input.value))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(JpOfficialError::new(
            JpOfficialErrorCode::IssuerNotFound,
            "issuer not found",
        ));
    }
    if candidates.len() > 1 {
        return Err(JpOfficialError::new(
            JpOfficialErrorCode::AmbiguousIssuer,
            "ambiguous issuer",
        ));
    }

    let c = &candidates[0];
    let aliases = c
        .aliases
        .iter()
        .filter_map(|a| {
            Some(IrIssuerAlias {
                market: IrMarket::Jp,
                source_id: input.source_id.clone(),
                identity_kind: parse_kind(&a.kind)?,
                value: a.value.clone(),
                provenance: a.provenance.clone(),
            })
        })
        .collect::<Vec<_>>();

    Ok(IrIssuerResolutionResult {
        issuer_key: IrIssuerKey {
            market: IrMarket::Jp,
            canonical_id: c.canonical_id.clone(),
        },
        aliases,
        provenance: IrIssuerIdentityProvenance {
            source_id: input.source_id.clone(),
            evidence: "fixture:ir_jp_official/issuers.json".into(),
        },
        confidence: IrIssuerConfidence(0.95),
    })
}
