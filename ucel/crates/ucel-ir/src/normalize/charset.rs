use super::errors::{IrNormalizationError, IrNormalizationReasonCode};

pub fn normalize_to_utf8(bytes: &[u8], declared: Option<&str>) -> Result<(String, Option<String>), IrNormalizationError> {
    let dec = declared.map(|v| v.to_ascii_lowercase());
    if matches!(dec.as_deref(), Some("utf-16") | Some("utf-16le") | Some("utf-16be") | Some("shift_jis") | Some("euc-jp")) {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::InvalidCharset, "unsupported charset in current normalizer"));
    }
    let txt = std::str::from_utf8(bytes)
        .map_err(|_| IrNormalizationError::new(IrNormalizationReasonCode::InvalidCharset, "bytes are not utf-8"))?
        .replace("\r\n", "\n");
    Ok((txt, Some("utf-8".to_string())))
}
