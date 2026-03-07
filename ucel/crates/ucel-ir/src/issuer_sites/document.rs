use super::errors::{IssuerSiteError, IssuerSiteErrorCode};
use crate::document::{IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest, IrDocumentListResponse};
use serde::Deserialize;
use ucel_core::{IrDocumentDescriptor, IrDocumentFamily, IrDocumentKey, IrIssuerKey, IrMarket};

#[derive(Debug, Deserialize)]
struct DocRow {
    source_id: String,
    market: String,
    source_document_id: String,
    issuer_canonical_id: String,
    family: String,
    title: String,
    language: Option<String>,
    published_at: Option<String>,
    source_page_url: String,
    access_pattern: String,
}

fn rows() -> Result<Vec<DocRow>, IssuerSiteError> {
    serde_json::from_str(include_str!("../../../../../ucel/fixtures/ir_issuer_sites/documents.json"))
        .map_err(|e| IssuerSiteError::new(IssuerSiteErrorCode::ParseFailed, e.to_string()))
}

fn parse_family(v: &str) -> IrDocumentFamily {
    match v {
        "earnings_release" => IrDocumentFamily::EarningsRelease,
        "earnings_presentation" => IrDocumentFamily::EarningsPresentation,
        "transcript" => IrDocumentFamily::Transcript,
        "press_release" => IrDocumentFamily::PressRelease,
        "integrated_report" => IrDocumentFamily::IntegratedReport,
        "sustainability_report" => IrDocumentFamily::SustainabilityReport,
        "governance_report" => IrDocumentFamily::GovernanceReport,
        "fact_sheet" => IrDocumentFamily::FactSheet,
        _ => IrDocumentFamily::MiscIrDocument,
    }
}

fn to_desc(d: &DocRow) -> IrDocumentDescriptor {
    let market = if d.market == "jp" { IrMarket::Jp } else { IrMarket::Us };
    IrDocumentDescriptor {
        key: IrDocumentKey { source_id: d.source_id.clone(), source_document_id: d.source_document_id.clone() },
        issuer_key: IrIssuerKey { market, canonical_id: d.issuer_canonical_id.clone() },
        source_id: d.source_id.clone(),
        market,
        family: parse_family(&d.family),
        title: d.title.clone(),
        language: d.language.clone(),
        filed_at: None,
        published_at: d.published_at.clone(),
    }
}

pub fn list(request: &IrDocumentListRequest) -> Result<IrDocumentListResponse, IssuerSiteError> {
    let docs = rows()?
        .into_iter()
        .filter(|d| d.source_id == request.source_id)
        .filter(|d| {
            if let Some(id) = &request.issuer_key {
                d.issuer_canonical_id == *id
            } else {
                true
            }
        })
        .map(|d| to_desc(&d))
        .collect::<Vec<_>>();
    Ok(IrDocumentListResponse { documents: docs, next_cursor: None })
}

pub fn detail(request: &IrDocumentDetailRequest) -> Result<IrDocumentDetailResponse, IssuerSiteError> {
    let row = rows()?
        .into_iter()
        .find(|d| d.source_id == request.source_id && d.source_document_id == request.key.source_document_id)
        .ok_or_else(|| IssuerSiteError::new(IssuerSiteErrorCode::DocumentNotFound, "document not found"))?;
    Ok(IrDocumentDetailResponse {
        detail: to_desc(&row),
        source_metadata: serde_json::json!({
            "source_page_url": row.source_page_url,
            "access_pattern": row.access_pattern,
            "provenance": "site_profile_html_or_feed",
        }),
    })
}
