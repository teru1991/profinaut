use super::errors::{IssuerSiteError, IssuerSiteErrorCode};
use crate::identity::{IrIssuerConfidence, IrIssuerIdentityProvenance, IrIssuerResolutionInput, IrIssuerResolutionResult};
use serde::Deserialize;
use ucel_core::{IrIssuerAlias, IrIssuerIdentityKind, IrIssuerKey, IrMarket};

#[derive(Debug, Deserialize)]
struct IssuerRow {
    source_id: String,
    market: String,
    canonical_id: String,
    name: String,
    aliases: Vec<String>,
    site_url: String,
    slug: String,
}

fn rows() -> Result<Vec<IssuerRow>, IssuerSiteError> {
    serde_json::from_str(include_str!("../../../../../ucel/fixtures/ir_issuer_sites/issuers.json"))
        .map_err(|e| IssuerSiteError::new(IssuerSiteErrorCode::ParseFailed, e.to_string()))
}

pub fn resolve(input: &IrIssuerResolutionInput) -> Result<IrIssuerResolutionResult, IssuerSiteError> {
    let market = match input.market { IrMarket::Jp => "jp", IrMarket::Us => "us" };
    let value = input.value.to_ascii_lowercase();
    let matches = rows()?
        .into_iter()
        .filter(|x| x.source_id == input.source_id && x.market == market)
        .filter(|x| {
            x.site_url.to_ascii_lowercase().contains(&value)
                || x.slug.to_ascii_lowercase() == value
                || x.aliases.iter().any(|a| a.to_ascii_lowercase() == value)
                || x.name.to_ascii_lowercase().contains(&value)
        })
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(IssuerSiteError::new(IssuerSiteErrorCode::IssuerSiteNotFound, "issuer not found"));
    }
    if matches.len() > 1 {
        return Err(IssuerSiteError::new(IssuerSiteErrorCode::AmbiguousSiteBinding, "ambiguous issuer site"));
    }
    let row = &matches[0];
    let market_enum = if row.market == "jp" { IrMarket::Jp } else { IrMarket::Us };
    let mut aliases = vec![
        IrIssuerAlias {
            market: market_enum,
            source_id: row.source_id.clone(),
            identity_kind: IrIssuerIdentityKind::UrlLike,
            value: row.site_url.clone(),
            provenance: "site_profile".into(),
        },
        IrIssuerAlias {
            market: market_enum,
            source_id: row.source_id.clone(),
            identity_kind: IrIssuerIdentityKind::IssuerSiteSlugLike,
            value: row.slug.clone(),
            provenance: "site_profile".into(),
        },
    ];
    for a in &row.aliases {
        aliases.push(IrIssuerAlias {
            market: market_enum,
            source_id: row.source_id.clone(),
            identity_kind: input.identity_kind,
            value: a.clone(),
            provenance: "official_metadata_seed".into(),
        });
    }
    Ok(IrIssuerResolutionResult {
        issuer_key: IrIssuerKey { market: market_enum, canonical_id: row.canonical_id.clone() },
        aliases,
        provenance: IrIssuerIdentityProvenance { source_id: row.source_id.clone(), evidence: format!("issuer-site:{}", row.site_url) },
        confidence: IrIssuerConfidence(0.82),
    })
}
