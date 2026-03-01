use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

use super::policy::DecimalPolicy;

pub fn deserialize_decimal_guarded<'de, D>(
    deserializer: D,
    policy: &DecimalPolicy,
) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyDecimal {
        Str(String),
        Num(serde_json::Number),
    }

    let any = AnyDecimal::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let s = match any {
        AnyDecimal::Str(s) => s,
        AnyDecimal::Num(n) => n.to_string(),
    };

    let d = s.parse::<Decimal>().map_err(serde::de::Error::custom)?;
    policy.guard().validate(d).map_err(serde::de::Error::custom)
}

pub fn deserialize_decimal_execution<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let policy = DecimalPolicy::for_execution_strict();
    deserialize_decimal_guarded(deserializer, &policy)
}

pub fn deserialize_decimal_balance<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let policy = DecimalPolicy::for_balance();
    deserialize_decimal_guarded(deserializer, &policy)
}

pub fn deserialize_decimal_observation<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let policy = DecimalPolicy::for_observation_relaxed();
    deserialize_decimal_guarded(deserializer, &policy)
}
