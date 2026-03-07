use super::errors::{IrNormalizationError, IrNormalizationReasonCode};

pub fn pdf_text_layer(raw: &[u8]) -> Result<String, IrNormalizationError> {
    if !raw.starts_with(b"%PDF-") {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::MalformedPdf, "missing PDF header"));
    }
    let mut out = String::new();
    let s = String::from_utf8_lossy(raw);
    let mut rest = s.as_ref();
    while let Some(pos) = rest.find('(') {
        rest = &rest[pos + 1..];
        if let Some(end) = rest.find(')') {
            out.push_str(&rest[..end]);
            out.push('\n');
            rest = &rest[end + 1..];
        } else { break; }
    }
    if out.trim().is_empty() {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::MalformedPdf, "no text layer found"));
    }
    Ok(out.trim().to_string())
}
