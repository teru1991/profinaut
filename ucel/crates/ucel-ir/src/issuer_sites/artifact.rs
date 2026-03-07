use super::access::{ensure_attachment_size, IssuerSitePolitenessPolicy};
use super::errors::{IssuerSiteError, IssuerSiteErrorCode};
use crate::artifact::{IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse};
use serde::Deserialize;
use ucel_core::{IrArtifactDescriptor, IrArtifactKey, IrArtifactKind, IrArtifactSource, IrDocumentKey};

#[derive(Debug, Deserialize)]
struct ArtifactRow {
    source_id: String,
    source_document_id: String,
    artifact_id: String,
    kind: String,
    content_type: String,
    size_bytes: u64,
    checksum_sha256: String,
    source_url: String,
    referring_page: String,
    encoding: Option<String>,
}

fn rows() -> Result<Vec<ArtifactRow>, IssuerSiteError> {
    serde_json::from_str(include_str!("../../../../../ucel/fixtures/ir_issuer_sites/artifacts.json"))
        .map_err(|e| IssuerSiteError::new(IssuerSiteErrorCode::ParseFailed, e.to_string()))
}

fn bytes_map() -> Result<serde_json::Map<String, serde_json::Value>, IssuerSiteError> {
    let v: serde_json::Value = serde_json::from_str(include_str!("../../../../../ucel/fixtures/ir_issuer_sites/artifact_bytes.json"))
        .map_err(|e| IssuerSiteError::new(IssuerSiteErrorCode::ParseFailed, e.to_string()))?;
    Ok(v.as_object().cloned().unwrap_or_default())
}

fn parse_kind(v: &str) -> Option<IrArtifactKind> {
    Some(match v {
        "html" => IrArtifactKind::Html,
        "pdf" => IrArtifactKind::Pdf,
        "xbrl" => IrArtifactKind::Xbrl,
        "ixbrl" => IrArtifactKind::Ixbrl,
        "xml" => IrArtifactKind::Xml,
        "txt" => IrArtifactKind::Txt,
        "csv" => IrArtifactKind::Csv,
        "zip" => IrArtifactKind::Zip,
        "json" => IrArtifactKind::Json,
        "rss" => IrArtifactKind::Rss,
        _ => return None,
    })
}

fn to_desc(a: &ArtifactRow) -> Result<IrArtifactDescriptor, IssuerSiteError> {
    Ok(IrArtifactDescriptor {
        key: IrArtifactKey {
            document: IrDocumentKey { source_id: a.source_id.clone(), source_document_id: a.source_document_id.clone() },
            artifact_id: a.artifact_id.clone(),
        },
        source_id: a.source_id.clone(),
        kind: parse_kind(&a.kind).ok_or_else(|| IssuerSiteError::new(IssuerSiteErrorCode::UnsupportedArtifactKind, &a.kind))?,
        content_type: Some(a.content_type.clone()),
        source: IrArtifactSource::LinkSource,
        checksum_sha256: Some(a.checksum_sha256.clone()),
        size_bytes: Some(a.size_bytes),
        encoding: a.encoding.clone(),
    })
}

pub fn list(request: &IrArtifactListRequest) -> Result<IrArtifactListResponse, IssuerSiteError> {
    let artifacts = rows()?
        .into_iter()
        .filter(|a| a.source_id == request.source_id && a.source_document_id == request.document_key.source_document_id)
        .map(|a| to_desc(&a))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IrArtifactListResponse { artifacts })
}

pub fn fetch(request: &IrArtifactFetchRequest, p: IssuerSitePolitenessPolicy) -> Result<IrArtifactFetchResponse, IssuerSiteError> {
    let row = rows()?
        .into_iter()
        .find(|a| a.source_id == request.source_id && a.artifact_id == request.artifact_key.artifact_id)
        .ok_or_else(|| IssuerSiteError::new(IssuerSiteErrorCode::ArtifactNotFound, "artifact not found"))?;
    if !(row.content_type.contains("html") || row.content_type.contains("pdf") || row.content_type.contains("xml") || row.content_type.contains("json") || row.content_type.contains("rss") || row.content_type.contains("zip") || row.content_type.contains("text")) {
        return Err(IssuerSiteError::new(IssuerSiteErrorCode::InvalidContentType, row.content_type));
    }
    ensure_attachment_size(row.size_bytes, p)?;
    let descriptor = to_desc(&row)?;
    let bytes_lookup = bytes_map()?;
    let text = bytes_lookup.get(&row.artifact_id).and_then(|v| v.as_str()).unwrap_or("").to_string();
    Ok(IrArtifactFetchResponse {
        metadata: descriptor,
        bytes: Some(text.as_bytes().to_vec()),
        text_candidate: Some(text),
        source_metadata: serde_json::json!({
            "source_url": row.source_url,
            "referring_page": row.referring_page,
            "size_bytes": row.size_bytes,
        }),
    })
}
