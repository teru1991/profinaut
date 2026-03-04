use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use ucel_diagnostics_core::{
    Contribution, ContributionContent, ContributionKind, DiagnosticsProvider, DiagnosticsRegistry,
    DiagnosticsRequest, ProviderMeta, RegistryLimits,
};
use ucel_transport::diagnostics::bundle::{build_support_bundle_tar_zst, read_tar_zst_entries};
use ucel_transport::diagnostics::limits::{BundleBuildError, BundleLimits};

struct P;
impl DiagnosticsProvider for P {
    fn meta(&self) -> ProviderMeta {
        ProviderMeta {
            provider_id: "alpha".into(),
            provider_version: "0.1.0".into(),
        }
    }
    fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
        vec![
            Contribution {
                provider_id: "alpha".into(),
                kind: ContributionKind::Text,
                path: "logs/tail.txt".into(),
                mime: "text/plain".into(),
                size_limit_bytes: 1024,
                content: ContributionContent::Text("hello\n".into()),
            },
            Contribution {
                provider_id: "alpha".into(),
                kind: ContributionKind::Json,
                path: "meta/info.json".into(),
                mime: "application/json".into(),
                size_limit_bytes: 1024,
                content: ContributionContent::Json(serde_json::json!({"k":"v"})),
            },
        ]
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[test]
fn builds_bundle_and_manifest_matches_archive() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(P)).unwrap();

    let req = DiagnosticsRequest::default();
    let limits = BundleLimits::default();
    let built = build_support_bundle_tar_zst(&r, &req, &limits).unwrap();

    let v: serde_json::Value = serde_json::from_slice(&built.manifest_json).unwrap();
    assert!(v.get("bundle_id").unwrap().as_str().unwrap().len() > 0);
    assert!(v.get("diag_semver").unwrap().as_str().unwrap().len() > 0);

    let entries = read_tar_zst_entries(&built.archive_bytes).unwrap();
    assert_eq!(entries[0].0, "manifest.json");

    let map: BTreeMap<String, Vec<u8>> = entries.into_iter().collect();
    let files = v.get("files").unwrap().as_array().unwrap();
    for f in files {
        let path = f.get("path").unwrap().as_str().unwrap();
        let size = f.get("size_bytes").unwrap().as_u64().unwrap();
        let sha = f.get("sha256").unwrap().as_str().unwrap();
        let bytes = map.get(path).unwrap();
        assert_eq!(bytes.len() as u64, size);
        assert_eq!(sha256_hex(bytes), sha);
    }

    assert!(map.contains_key("meta/diag_semver.txt"));
    assert!(map.contains_key("logs/tail.txt"));
    assert!(map.contains_key("meta/info.json"));
}

#[test]
fn is_deterministic_for_same_input() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(P)).unwrap();
    let req = DiagnosticsRequest::default();
    let limits = BundleLimits::default();

    let a = build_support_bundle_tar_zst(&r, &req, &limits).unwrap();
    let b = build_support_bundle_tar_zst(&r, &req, &limits).unwrap();

    assert_eq!(a.bundle_id, b.bundle_id);
    assert_eq!(a.manifest_json, b.manifest_json);
    assert_eq!(a.archive_bytes, b.archive_bytes);
}

#[test]
fn rejects_total_size_exceeded() {
    struct Big;
    impl DiagnosticsProvider for Big {
        fn meta(&self) -> ProviderMeta {
            ProviderMeta {
                provider_id: "big".into(),
                provider_version: "0.1.0".into(),
            }
        }
        fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
            let s = "a".repeat(2048);
            vec![Contribution {
                provider_id: "big".into(),
                kind: ContributionKind::Text,
                path: "logs/big.txt".into(),
                mime: "text/plain".into(),
                size_limit_bytes: 10_000_000,
                content: ContributionContent::Text(s),
            }]
        }
    }

    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(Big)).unwrap();

    let req = DiagnosticsRequest::default();
    let mut limits = BundleLimits::default();
    limits.max_total_bytes = 512;
    let err = build_support_bundle_tar_zst(&r, &req, &limits).unwrap_err();
    match err {
        BundleBuildError::TotalSizeExceeded(_) => {}
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_invalid_path() {
    struct Bad;
    impl DiagnosticsProvider for Bad {
        fn meta(&self) -> ProviderMeta {
            ProviderMeta {
                provider_id: "bad".into(),
                provider_version: "0.1.0".into(),
            }
        }
        fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
            vec![Contribution {
                provider_id: "bad".into(),
                kind: ContributionKind::Text,
                path: "bad path.txt".into(),
                mime: "text/plain".into(),
                size_limit_bytes: 1024,
                content: ContributionContent::Text("no".into()),
            }]
        }
    }

    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(Bad)).unwrap();

    let req = DiagnosticsRequest::default();
    let limits = BundleLimits::default();
    let err = build_support_bundle_tar_zst(&r, &req, &limits).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("invalid path") || msg.contains("InvalidPath"));
}
