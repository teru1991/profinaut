use super::errors::{IrNormalizationError, IrNormalizationReasonCode};

pub fn normalize_rss(raw: &str) -> Result<String, IrNormalizationError> {
    if !raw.to_ascii_lowercase().contains("<rss") {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::ParseFailed, "not rss"));
    }
    Ok(super::xml::xml_to_text(raw))
}
