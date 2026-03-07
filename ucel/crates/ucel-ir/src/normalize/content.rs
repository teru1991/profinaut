use super::errors::{IrNormalizationError, IrNormalizationReasonCode};
use ucel_core::{IrNormalizationProvenance, IrNormalizationSchemaVersion, IrNormalizationSupport, IrNormalizedContent, IrNormalizedFormat};
use crate::artifact::IrArtifactFetchResponse;

pub fn assemble(
    fetch: &IrArtifactFetchResponse,
    format: IrNormalizedFormat,
    text: String,
    sections: Vec<ucel_core::IrNormalizedSection>,
    tables: Vec<ucel_core::IrNormalizedTable>,
    attachments: Vec<ucel_core::IrNormalizedAttachment>,
    charset: Option<String>,
    support_level: IrNormalizationSupport,
) -> Result<IrNormalizedContent, IrNormalizationError> {
    if sections.is_empty() {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::ProvenanceLost, "sections cannot be empty"));
    }
    Ok(IrNormalizedContent {
        document_key: fetch.metadata.key.document.clone(),
        artifact_key: fetch.metadata.key.clone(),
        normalization_schema_version: IrNormalizationSchemaVersion::default(),
        normalized_format: format,
        normalized_text: text,
        sections,
        tables,
        extracted_attachments: attachments,
        language_hints: vec![],
        charset,
        provenance: IrNormalizationProvenance { source_type: Some("artifact".into()), source_ref: Some(fetch.metadata.key.artifact_id.clone()), context_ref: None, extra: Default::default() },
        support_level,
    })
}
