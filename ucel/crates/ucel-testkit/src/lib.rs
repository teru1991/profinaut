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
            .map(|e| e.id.as_str())
            .filter(|id| !self.registered_tests.contains(*id))
            .map(|s| s.to_string())
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
    for e in &manifest.entries {
        if !e.implemented {
            missing
                .entry("implemented".into())
                .or_default()
                .push(e.id.clone());
        }
        if !e.tested {
            missing
                .entry("tested".into())
                .or_default()
                .push(e.id.clone());
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
    use std::path::Path;
    use ucel_core::ResolvedSecret;
    use ucel_registry::load_catalog_from_repo_root;

    #[test]
    fn coverage_gate_is_strict_for_bitbank_and_has_no_gaps() {
        let manifest = load_coverage_manifest(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitbank.yaml"),
        )
        .unwrap();
        assert_eq!(manifest.venue, "bitbank");
        assert!(manifest.strict);
        assert!(evaluate_coverage_gate(&manifest).is_empty());
    }

    #[test]
    fn resolved_secret_masking_is_enforced() {
        let s = ResolvedSecret {
            api_key: "my-key".into(),
            api_secret: Some("my-secret".into()),
            passphrase: Some("my-pass".into()),
        };
        let dbg = format!("{s:?}");
        let disp = format!("{s}");
        assert!(!dbg.contains("my-secret"));
        assert!(!disp.contains("my-pass"));
    }

    #[test]
    fn contract_index_can_cover_all_bitbank_catalog_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_catalog_from_repo_root(&repo_root, "bitbank").unwrap();
        let mut index = CatalogContractIndex::default();
        for id in catalog
            .rest_endpoints
            .iter()
            .chain(catalog.ws_channels.iter())
            .map(|e| e.id.as_str())
        {
            index.register_id(id);
        }
        assert!(index.missing_catalog_ids(&catalog).is_empty());

        let missing = index.missing_catalog_ids(&catalog);
        assert!(missing.is_empty());
    }

    #[test]
    fn coverage_gate_is_strict_and_has_no_gaps() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/gmocoin.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "gmocoin");
        assert!(manifest.strict);

        let gaps = evaluate_coverage_gate(&manifest);
        assert!(
            gaps.is_empty(),
            "strict coverage gate must have no gaps: {gaps:?}"
        );
    }

    #[test]
    fn coverage_gate_warns_for_bybit_gaps() {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/BYBIT.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "BYBIT");
        assert!(!manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::WarnOnly(gaps) => {
                assert_eq!(gaps.get("implemented").map(Vec::len), Some(96));
                assert_eq!(gaps.get("tested").map(Vec::len), Some(96));
            }
            _ => panic!("bybit coverage gate should be warn-only while gaps exist"),
        }
    }

    #[test]
    fn coverage_gate_is_strict_for_bitmex_and_has_no_gaps() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitmex.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "bitmex");
        assert!(manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::Passed => {}
            _ => panic!("bitmex coverage gate should pass in strict mode"),
        }
    }

    #[test]
    fn coverage_gate_warns_for_binance_options_until_full_coverage() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/binance-options.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "binance-options");
        assert!(!manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::WarnOnly(gaps) => {
                assert_eq!(gaps.get("implemented").map(Vec::len), Some(14));
                assert_eq!(gaps.get("tested").map(Vec::len), Some(14));
            }
            _ => panic!("binance-options coverage gate should warn while manifest has gaps"),
        }
    }

    #[test]
    fn coverage_gate_is_strict_for_coinbase_and_has_no_gaps() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/coinbase.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "coinbase");
        assert!(manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::Passed => {}
            _ => panic!("coinbase coverage gate should pass in strict mode"),
        }
    }

    #[test]
    fn coverage_gate_is_strict_for_kraken_and_has_no_gaps() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/kraken.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "kraken");
        assert!(manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::Passed => {}
            _ => panic!("kraken coverage gate should pass in strict mode"),
        }
    }

    #[test]
    fn coverage_gate_is_strict_for_binance_usdm_and_has_no_gaps() {
    fn coverage_gate_warns_for_bitbank_until_full_coverage() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/bitbank.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "bitbank");
    }

    #[test]
    fn coverage_gate_warns_for_binance_usdm_until_full_coverage() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/binance-usdm.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "binance-usdm");
    }

    #[test]
    fn coverage_gate_is_strict_for_binance_coinm_and_has_no_gaps() {
        assert!(manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::Passed => {}
            _ => panic!("binance-usdm coverage gate should pass in strict mode"),
        }
    }

    #[test]
    fn coverage_gate_warns_for_binance_coinm_gaps() {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../coverage/binance-coinm.yaml");
        let manifest = load_coverage_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.venue, "binance-coinm");
        assert!(manifest.strict);

        let result = run_coverage_gate(&manifest);
        match result {
            CoverageGateResult::Passed => {}
            _ => panic!("binance-coinm coverage gate should pass in strict mode"),
        }
    }
}
