use super::errors::{JpOfficialError, JpOfficialErrorCode};
use crate::document::{
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse,
};
use serde::Deserialize;
use ucel_core::{IrDocumentDescriptor, IrDocumentFamily, IrDocumentKey, IrIssuerKey, IrMarket};

#[derive(Debug, Deserialize, Clone)]
struct FixtureDocument {
    source_id: String,
    source_document_id: String,
    canonical_issuer_id: String,
    family: String,
    title: String,
    language: String,
    filed_at: String,
    published_at: String,
    access_pattern: String,
}

fn docs() -> Result<Vec<FixtureDocument>, JpOfficialError> {
    serde_json::from_str(include_str!(
        "../../../../../ucel/fixtures/ir_jp_official/documents.json"
    ))
    .map_err(|e| JpOfficialError::new(JpOfficialErrorCode::ParseFailed, e.to_string()))
}

fn parse_family(v: &str) -> Option<IrDocumentFamily> {
    Some(match v {
        "statutory_annual" => IrDocumentFamily::StatutoryAnnual,
        "timely_disclosure" => IrDocumentFamily::TimelyDisclosure,
        _ => return None,
    })
}

fn to_descriptor(d: &FixtureDocument) -> Result<IrDocumentDescriptor, JpOfficialError> {
    Ok(IrDocumentDescriptor {
        key: IrDocumentKey {
            source_id: d.source_id.clone(),
            source_document_id: d.source_document_id.clone(),
        },
        issuer_key: IrIssuerKey {
            market: IrMarket::Jp,
            canonical_id: d.canonical_issuer_id.clone(),
        },
        source_id: d.source_id.clone(),
        market: IrMarket::Jp,
        family: parse_family(&d.family).ok_or_else(|| {
            JpOfficialError::new(JpOfficialErrorCode::ParseFailed, "unknown family")
        })?,
        title: d.title.clone(),
        language: Some(d.language.clone()),
        filed_at: Some(d.filed_at.clone()),
        published_at: Some(d.published_at.clone()),
    })
}

pub fn list(request: &IrDocumentListRequest) -> Result<IrDocumentListResponse, JpOfficialError> {
    let rows = docs()?
        .into_iter()
        .filter(|d| d.source_id == request.source_id)
        .map(|d| to_descriptor(&d))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IrDocumentListResponse {
        documents: rows,
        next_cursor: None,
    })
}

pub fn detail(
    request: &IrDocumentDetailRequest,
) -> Result<IrDocumentDetailResponse, JpOfficialError> {
    let row = docs()?
        .into_iter()
        .find(|d| {
            d.source_id == request.source_id
                && d.source_document_id == request.key.source_document_id
        })
        .ok_or_else(|| {
            JpOfficialError::new(JpOfficialErrorCode::DocumentNotFound, "document not found")
        })?;
    Ok(IrDocumentDetailResponse {
        detail: to_descriptor(&row)?,
        source_metadata: serde_json::json!({"access_pattern": row.access_pattern, "source_document_id": row.source_document_id}),
    })
}
