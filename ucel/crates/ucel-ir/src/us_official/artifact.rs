use super::access::{ensure_attachment_size, UsPolitenessPolicy};
use super::errors::{UsOfficialError, UsOfficialErrorCode};
use crate::artifact::{
    IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse,
};
use serde::Deserialize;
use ucel_core::{
    IrArtifactDescriptor, IrArtifactKey, IrArtifactKind, IrArtifactSource, IrDocumentKey,
};

#[derive(Debug, Deserialize, Clone)]
struct FixtureArtifact {
    source_id: String,
    source_document_id: String,
    artifact_id: String,
    kind: String,
    content_type: Option<String>,
    size_bytes: u64,
    encoding: Option<String>,
    source_url: String,
    checksum_sha256: String,
}

fn rows() -> Result<Vec<FixtureArtifact>, UsOfficialError> {
    serde_json::from_str(include_str!(
        "../../../../../ucel/fixtures/ir_us_official/artifacts.json"
    ))
    .map_err(|e| UsOfficialError::new(UsOfficialErrorCode::ParseFailed, e.to_string()))
}

fn bytes_map() -> Result<serde_json::Map<String, serde_json::Value>, UsOfficialError> {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../ucel/fixtures/ir_us_official/artifact_bytes.json"
    ))
    .map_err(|e| UsOfficialError::new(UsOfficialErrorCode::ParseFailed, e.to_string()))?;
    Ok(v.as_object().cloned().unwrap_or_default())
}

fn parse_kind(v: &str) -> Option<IrArtifactKind> {
    Some(match v {
        "json" => IrArtifactKind::Json,
        "pdf" => IrArtifactKind::Pdf,
        "html" => IrArtifactKind::Html,
        "xbrl" => IrArtifactKind::Xbrl,
        "ixbrl" => IrArtifactKind::Ixbrl,
        "xml" => IrArtifactKind::Xml,
        "txt" => IrArtifactKind::Txt,
        "zip" => IrArtifactKind::Zip,
        _ => return None,
    })
}

fn to_descriptor(a: &FixtureArtifact) -> Result<IrArtifactDescriptor, UsOfficialError> {
    Ok(IrArtifactDescriptor {
        key: IrArtifactKey {
            document: IrDocumentKey {
                source_id: a.source_id.clone(),
                source_document_id: a.source_document_id.clone(),
            },
            artifact_id: a.artifact_id.clone(),
        },
        source_id: a.source_id.clone(),
        kind: parse_kind(&a.kind).ok_or_else(|| {
            UsOfficialError::new(UsOfficialErrorCode::UnsupportedArtifactKind, &a.kind)
        })?,
        content_type: a.content_type.clone(),
        source: IrArtifactSource::LinkSource,
        checksum_sha256: Some(a.checksum_sha256.clone()),
        size_bytes: Some(a.size_bytes),
        encoding: a.encoding.clone(),
    })
}

pub fn list(request: &IrArtifactListRequest) -> Result<IrArtifactListResponse, UsOfficialError> {
    let artifacts = rows()?
        .into_iter()
        .filter(|x| {
            x.source_id == request.source_id
                && x.source_document_id == request.document_key.source_document_id
        })
        .map(|x| to_descriptor(&x))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IrArtifactListResponse { artifacts })
}

pub fn fetch(
    request: &IrArtifactFetchRequest,
    policy: UsPolitenessPolicy,
) -> Result<IrArtifactFetchResponse, UsOfficialError> {
    let row = rows()?
        .into_iter()
        .find(|x| {
            x.source_id == request.source_id && x.artifact_id == request.artifact_key.artifact_id
        })
        .ok_or_else(|| {
            UsOfficialError::new(UsOfficialErrorCode::ArtifactNotFound, "artifact not found")
        })?;
    if let Some(ct) = &row.content_type {
        if !(ct.contains("json")
            || ct.contains("pdf")
            || ct.contains("html")
            || ct.contains("xml")
            || ct.contains("text")
            || ct.contains("zip"))
        {
            return Err(UsOfficialError::new(
                UsOfficialErrorCode::InvalidContentType,
                ct.clone(),
            ));
        }
    }
    ensure_attachment_size(row.size_bytes, policy)?;
    let descriptor = to_descriptor(&row)?;
    let bytes_lookup = bytes_map()?;
    let text = bytes_lookup
        .get(&row.artifact_id)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    Ok(IrArtifactFetchResponse {
        metadata: descriptor,
        bytes: Some(text.as_bytes().to_vec()),
        text_candidate: Some(text.clone()),
        source_metadata: serde_json::json!({
            "source_url": row.source_url,
            "size_bytes": row.size_bytes,
            "user_agent": policy.user_agent
        }),
    })
}
