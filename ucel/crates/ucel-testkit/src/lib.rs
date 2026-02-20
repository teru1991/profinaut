use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::{fs, path::Path};
use ucel_core::ErrorCode;
use ucel_registry::ExchangeCatalog;
use ucel_transport::{HealthLevel, HealthStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    RestRespond {
        path: String,
        status: u16,
        body: String,
        headers: Vec<(String, String)>,
    },
    RestRateLimit429 {
        path: String,
        retry_after_ms: u64,
    },
    WsAccept,
    WsSendJson(String),
    WsDropConnection,
    SleepMs(u64),
    InjectOrderBookGap,
    InjectOutOfOrder,
    InjectDuplicate,
    ExpectErrorCode(ErrorCode),
    ExpectMetricInc(String),
    ExpectDegraded,
}

#[derive(Debug, Default)]
pub struct RestMockServer {
    pub responses: VecDeque<(u16, String)>,
}
impl RestMockServer {
    pub fn enqueue(&mut self, status: u16, body: impl Into<String>) {
        self.responses.push_back((status, body.into()));
    }
    pub fn next_response(&mut self) -> Option<(u16, String)> {
        self.responses.pop_front()
    }
}

#[derive(Debug, Default)]
pub struct WsMockServer {
    pub accepted: bool,
    pub events: VecDeque<String>,
    pub dropped: bool,
}
impl WsMockServer {
    pub fn accept(&mut self) {
        self.accepted = true;
    }
    pub fn send_json(&mut self, msg: impl Into<String>) {
        self.events.push_back(msg.into());
    }
    pub fn drop_connection(&mut self) {
        self.dropped = true;
    }
}

pub fn degraded_health(reason: &str, code: ErrorCode) -> HealthStatus {
    HealthStatus {
        level: HealthLevel::Degraded,
        degraded_reason: Some(reason.into()),
        last_success_ts: None,
        last_error_code: Some(code),
    }
}

#[derive(Debug, Default)]
pub struct CatalogContractIndex {
    registered_tests: HashSet<String>,
}
impl CatalogContractIndex {
    pub fn register_id(&mut self, id: &str) {
        self.registered_tests.insert(id.to_string());
    }
    pub fn missing_catalog_ids(&self, catalog: &ExchangeCatalog) -> Vec<String> {
        catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
            .filter(|entry| !self.registered_tests.contains(&entry.id))
            .map(|entry| entry.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageManifest {
    pub venue: String,
    pub strict: bool,
    pub entries: Vec<CoverageEntry>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct CoverageEntry {
    pub id: String,
    pub implemented: bool,
    pub tested: bool,
}

pub fn load_coverage_manifest(path: &Path) -> Result<CoverageManifest, Box<dyn std::error::Error>> {
    Ok(serde_yaml::from_str(&fs::read_to_string(path)?)?)
}

pub fn evaluate_coverage_gate(manifest: &CoverageManifest) -> HashMap<String, Vec<String>> {
    let mut missing: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &manifest.entries {
        if !entry.implemented {
            missing
                .entry("implemented".into())
                .or_default()
                .push(entry.id.clone());
        }
        if !entry.tested {
            missing
                .entry("tested".into())
                .or_default()
                .push(entry.id.clone());
        }
    }
    missing
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageGateResult {
    Passed,
    WarnOnly(HashMap<String, Vec<String>>),
    Failed(HashMap<String, Vec<String>>),
}

pub fn run_coverage_gate(manifest: &CoverageManifest) -> CoverageGateResult {
    let gaps = evaluate_coverage_gate(manifest);
    if gaps.is_empty() {
        CoverageGateResult::Passed
    } else if manifest.strict {
        CoverageGateResult::Failed(gaps)
    } else {
        CoverageGateResult::WarnOnly(gaps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_gate_returns_failed_when_strict_with_gaps() {
        let manifest = CoverageManifest {
            venue: "x".into(),
            strict: true,
            entries: vec![CoverageEntry {
                id: "a".into(),
                implemented: false,
                tested: false,
            }],
        };
        assert!(matches!(
            run_coverage_gate(&manifest),
            CoverageGateResult::Failed(_)
        ));
    }

    #[test]
    fn coverage_gate_returns_warn_only_when_non_strict_with_gaps() {
        let manifest = CoverageManifest {
            venue: "x".into(),
            strict: false,
            entries: vec![CoverageEntry {
                id: "a".into(),
                implemented: false,
                tested: false,
            }],
        };
        assert!(matches!(
            run_coverage_gate(&manifest),
            CoverageGateResult::WarnOnly(_)
        ));
    }
}
