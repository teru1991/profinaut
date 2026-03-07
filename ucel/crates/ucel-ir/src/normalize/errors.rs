use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrNormalizationReasonCode {
    UnknownFormat,
    InvalidCharset,
    ParseFailed,
    OversizedArtifact,
    InvalidArchive,
    UnsupportedNestedArchive,
    MalformedXbrl,
    MalformedHtml,
    MalformedPdf,
    TableExtractionFailed,
    ProvenanceLost,
}

#[derive(Debug, Clone)]
pub struct IrNormalizationError {
    pub reason: IrNormalizationReasonCode,
    pub message: String,
    pub metadata: BTreeMap<String, String>,
}

impl IrNormalizationError {
    pub fn new(reason: IrNormalizationReasonCode, message: impl Into<String>) -> Self {
        Self { reason, message: message.into(), metadata: BTreeMap::new() }
    }
}
