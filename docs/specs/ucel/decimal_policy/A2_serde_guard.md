UCEL Decimal Policy (A2) — Serde Guard (SSOT)

Rust 雛形（新規：ucel/crates/ucel-core/src/decimal/serde.rs）

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

use super::policy::DecimalPolicy;

/// Deserialize Decimal from string/number then apply UCEL guard.
/// This is SSOT: input rejection must be centralized.
pub fn deserialize_decimal_guarded<'de, D>(
    deserializer: D,
    policy: &DecimalPolicy,
) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    // Accept either string or number
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyDecimal {
        Str(String),
        Num(serde_json::Number),
    }

    let any = AnyDecimal::deserialize(deserializer).map_err(serde::de::Error::custom)?;

    let d = match any {
        AnyDecimal::Str(s) => s.parse::<Decimal>().map_err(serde::de::Error::custom)?,
        AnyDecimal::Num(n) => {
            // Convert via string to avoid float rounding surprises.
            let s = n.to_string();
            s.parse::<Decimal>().map_err(serde::de::Error::custom)?
        }
    };

    policy
        .guard()
        .validate(d)
        .map_err(serde::de::Error::custom)
}

/// Convenience: global default policy guarded decimal.
/// Use only if your crate truly wants default SSOT policy.
/// Prefer injecting policy explicitly.
pub fn deserialize_decimal_guarded_default<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let policy = DecimalPolicy::default();
    deserialize_decimal_guarded(deserializer, &policy)
}

使用例（docs-only）

// #[derive(Deserialize)]
// struct WsMsg {
//   #[serde(deserialize_with = "ucel_core::decimal::serde::deserialize_decimal_guarded_default")]
//   price: Decimal,
// }
