use rust_decimal::Decimal as RustDecimal;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Canonical numeric type for price/qty/money.
/// `serde-with-float` enables decoding JSON floats while preserving Decimal internally.
pub type Decimal = RustDecimal;

/// Schema version stored as SemVer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(pub Version);

impl SchemaVersion {
    pub fn parse(s: &str) -> Result<Self, String> {
        Version::parse(s)
            .map(SchemaVersion)
            .map_err(|e| e.to_string())
    }
}

impl FromStr for SchemaVersion {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SchemaVersion::parse(s)
    }
}

/// Buy/Sell side with forward compatible Unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
    #[serde(other)]
    Unknown,
}

/// Order type with forward compatible Unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    #[serde(other)]
    Unknown,
}

/// Order status with forward compatible Unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    #[serde(other)]
    Unknown,
}
