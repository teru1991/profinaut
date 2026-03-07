use super::errors::{IrNormalizationError, IrNormalizationReasonCode};

pub fn normalize_json(raw: &str) -> Result<String, IrNormalizationError> {
    let v: serde_json::Value = serde_json::from_str(raw)
        .map_err(|_| IrNormalizationError::new(IrNormalizationReasonCode::ParseFailed, "invalid json"))?;
    Ok(serde_json::to_string_pretty(&v).unwrap_or_default())
}
