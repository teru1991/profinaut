use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Provider metadata for registry bookkeeping.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderMeta {
    pub provider_id: String,
    pub provider_version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsRequest {
    pub run_id: Option<String>,
    pub incident_id: Option<String>,
    pub trace_id: Option<String>,
    pub change_id: Option<String>,
    pub approval_id: Option<String>,
    pub repair_id: Option<String>,
    pub silence_id: Option<String>,
    pub severity: Option<String>,
    /// Allow deep capture only when break-glass is active (policy enforced by upper layers).
    pub allow_deep: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionKind {
    Json,
    Text,
    Binary,
    Meta,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContributionContent {
    Json(Value),
    Text(String),
    /// For binary: base64 string (policy may forbid later)
    Base64(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub provider_id: String,
    pub kind: ContributionKind,
    /// Relative path inside bundle root. Must be normalized and safe.
    pub path: String,
    #[serde(default)]
    pub mime: String,
    #[serde(default = "default_size_limit")]
    pub size_limit_bytes: u64,
    pub content: ContributionContent,
}

fn default_size_limit() -> u64 {
    1_048_576 // 1 MiB default; upper layers may override
}

impl Contribution {
    /// Deterministic sort key: (provider_id, path, kind)
    pub fn sort_key(&self) -> (&str, &str, u8) {
        let k = match self.kind {
            ContributionKind::Json => 0,
            ContributionKind::Text => 1,
            ContributionKind::Meta => 2,
            ContributionKind::Binary => 3,
        };
        (self.provider_id.as_str(), self.path.as_str(), k)
    }
}

/// Contract boundary trait: domain diagnostics contribution provider.
/// Object-safe (no generics in signatures).
pub trait DiagnosticsProvider: Send + Sync {
    fn meta(&self) -> ProviderMeta;

    /// Must return a stable set of contributions for a given request.
    /// Upper layers will enforce deterministic ordering and limits.
    fn contributions(&self, req: &DiagnosticsRequest) -> Vec<Contribution>;
}
