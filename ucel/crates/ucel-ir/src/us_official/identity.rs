use super::errors::{UsOfficialError, UsOfficialErrorCode};
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

fn issuers() -> Result<Vec<FixtureIssuer>, UsOfficialError> {
    serde_json::from_str(include_str!(
        "../../../../../ucel/fixtures/ir_us_official/issuers.json"
    ))
    .map_err(|e| UsOfficialError::new(UsOfficialErrorCode::ParseFailed, e.to_string()))
}

fn parse_kind(kind: &str) -> Option<IrIssuerIdentityKind> {
    Some(match kind {
        "us_cik_like" => IrIssuerIdentityKind::UsCikLike,
        "ticker_like" => IrIssuerIdentityKind::TickerLike,
        "exchange_ticker_like" => IrIssuerIdentityKind::ExchangeTickerLike,
        "url_like" => IrIssuerIdentityKind::UrlLike,
        _ => return None,
    })
}

pub fn resolve(
    input: &IrIssuerResolutionInput,
) -> Result<IrIssuerResolutionResult, UsOfficialError> {
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
        return Err(UsOfficialError::new(
            UsOfficialErrorCode::IssuerNotFound,
            "issuer not found",
        ));
    }
    if candidates.len() > 1 {
        return Err(UsOfficialError::new(
            UsOfficialErrorCode::AmbiguousIssuer,
            "ambiguous issuer",
        ));
    }

    let c = &candidates[0];
    let aliases = c
        .aliases
        .iter()
        .filter_map(|a| {
            Some(IrIssuerAlias {
                market: IrMarket::Us,
                source_id: input.source_id.clone(),
                identity_kind: parse_kind(&a.kind)?,
                value: a.value.clone(),
                provenance: a.provenance.clone(),
            })
        })
        .collect::<Vec<_>>();

    Ok(IrIssuerResolutionResult {
        issuer_key: IrIssuerKey {
            market: IrMarket::Us,
            canonical_id: c.canonical_id.clone(),
        },
        aliases,
        provenance: IrIssuerIdentityProvenance {
            source_id: input.source_id.clone(),
            evidence: "fixture:ir_us_official/issuers.json".into(),
        },
        confidence: IrIssuerConfidence(0.96),
    })
}
