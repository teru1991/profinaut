pub mod charset;
pub mod content;
pub mod csv;
pub mod detect;
pub mod errors;
pub mod html;
pub mod json;
pub mod pdf;
pub mod rss;
pub mod safety;
pub mod sections;
pub mod tables;
pub mod text;
pub mod xbrl;
pub mod xml;
pub mod zip;

use crate::artifact::IrArtifactFetchResponse;
use errors::{IrNormalizationError, IrNormalizationReasonCode};
use ucel_core::{IrNormalizationSupport, IrNormalizedContent, IrNormalizedFormat};

pub fn normalize_artifact(fetch: &IrArtifactFetchResponse) -> Result<IrNormalizedContent, IrNormalizationError> {
    let bytes = fetch.bytes.as_deref().unwrap_or_default();
    let fmt = detect::detect_format(&fetch.metadata, bytes)
        .ok_or_else(|| IrNormalizationError::new(IrNormalizationReasonCode::UnknownFormat, "unable to detect artifact format"))?;
    normalize_artifact_with_format(fetch, fmt)
}

pub fn normalize_artifact_with_format(fetch: &IrArtifactFetchResponse, fmt: IrNormalizedFormat) -> Result<IrNormalizedContent, IrNormalizationError> {
    let bytes = fetch.bytes.as_deref().unwrap_or_default();
    let raw_text = if let Some(t) = &fetch.text_candidate { t.clone() } else { String::from_utf8_lossy(bytes).to_string() };
    let (txt, charset) = charset::normalize_to_utf8(raw_text.as_bytes(), fetch.metadata.encoding.as_deref())?;
    let normalized_text = match fmt {
        IrNormalizedFormat::Html | IrNormalizedFormat::Ixbrl => html::html_to_text(&txt),
        IrNormalizedFormat::Xml => xml::xml_to_text(&txt),
        IrNormalizedFormat::Xbrl => xbrl::xbrl_to_text(&txt),
        IrNormalizedFormat::Pdf => pdf::pdf_text_layer(bytes)?,
        IrNormalizedFormat::Txt => text::normalize_text(&txt),
        IrNormalizedFormat::Csv => csv::normalize_csv(&txt),
        IrNormalizedFormat::Json => json::normalize_json(&txt)?,
        IrNormalizedFormat::Rss => rss::normalize_rss(&txt)?,
        IrNormalizedFormat::Zip => "[zip archive]".to_string(),
    };
    let sections = sections::sections_from_text(&normalized_text);
    let tables = if matches!(fmt, IrNormalizedFormat::Csv) { vec![tables::csv_to_table(&normalized_text)] } else { vec![] };
    let attachments = if matches!(fmt, IrNormalizedFormat::Zip) { zip::unpack_zip(bytes, safety::IrUnpackPolicy::default())? } else { vec![] };
    content::assemble(fetch, fmt, normalized_text, sections, tables, attachments, charset, IrNormalizationSupport::Supported)
}
