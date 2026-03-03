use crate::execution::{SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IdempotencyKey は "入口で必須"。
/// - 既存の運用で Idempotency-Key header 等へ転写できるよう、文字列として保持
/// - 安定生成も提供（intent から hash）
/// - ただし、運用上は "外部注入" を第一級で許す（再送/復旧の自由度）
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(pub String);

impl IdempotencyKey {
    pub fn parse(s: impl Into<String>) -> SdkExecutionResult<Self> {
        let s = s.into();
        let t = s.trim();
        if t.is_empty() {
            return Err(SdkExecutionError::new(
                SdkExecutionErrorCode::InvalidInput,
                "idempotency empty",
            ));
        }
        if t.len() < 16 || t.len() > 128 {
            return Err(SdkExecutionError::new(
                SdkExecutionErrorCode::InvalidInput,
                "idempotency length out of range",
            ));
        }
        // 文字種を緩めすぎるとログ/監査で事故るので、ASCII printable のみに制限
        if !t.bytes().all(|b| (0x21..=0x7E).contains(&b)) {
            return Err(SdkExecutionError::new(
                SdkExecutionErrorCode::InvalidInput,
                "idempotency must be ascii printable",
            ));
        }
        Ok(Self(t.to_string()))
    }

    pub fn random_uuid() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// intent の JSON を canonical-ish にして blake3 でハッシュ。
    /// "順序が揺れる map" は types で BTreeMap を使っているため安定する。
    pub fn derive_from_intent(intent: &crate::execution::OrderIntent) -> Self {
        let bytes = serde_json::to_vec(intent).unwrap_or_default();
        let h = blake3::hash(&bytes);
        // 32 bytes -> hex 64 chars（固定長）
        Self(format!("{}", h.to_hex()))
    }
}
