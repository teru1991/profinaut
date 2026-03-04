use ucel_diagnostics_core::{
    Contribution, ContributionContent, ContributionKind, DiagnosticsProvider, DiagnosticsRegistry,
    DiagnosticsRequest, ProviderMeta, RegistryLimits,
};
use ucel_transport::diagnostics::bundle::{build_support_bundle_tar_zst, read_tar_zst_entries};
use ucel_transport::diagnostics::limits::BundleLimits;

struct Leaky;
impl DiagnosticsProvider for Leaky {
    fn meta(&self) -> ProviderMeta {
        ProviderMeta {
            provider_id: "leaky".into(),
            provider_version: "0.1.0".into(),
        }
    }
    fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
        vec![Contribution {
            provider_id: "leaky".into(),
            kind: ContributionKind::Text,
            path: "logs/http.txt".into(),
            mime: "text/plain".into(),
            size_limit_bytes: 4096,
            content: ContributionContent::Text(
                "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.aaaaaaaaaaa.bbbbbbbbbbb\n"
                    .into(),
            ),
        }]
    }
}

struct Clean;
impl DiagnosticsProvider for Clean {
    fn meta(&self) -> ProviderMeta {
        ProviderMeta {
            provider_id: "clean".into(),
            provider_version: "0.1.0".into(),
        }
    }
    fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
        vec![Contribution {
            provider_id: "clean".into(),
            kind: ContributionKind::Json,
            path: "meta/event.json".into(),
            mime: "application/json".into(),
            size_limit_bytes: 4096,
            content: ContributionContent::Json(
                serde_json::json!({"token":"abcd1234EFGH5678IJKL9012MNOP3456"}),
            ),
        }]
    }
}

#[test]
fn leaky_bundle_is_rejected_fail_closed() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(Leaky)).unwrap();

    let req = DiagnosticsRequest::default();
    let limits = BundleLimits::default();
    let err = build_support_bundle_tar_zst(&r, &req, &limits).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("redaction") || msg.contains("fail-closed"));
}

#[test]
fn clean_bundle_succeeds_and_redacts_json_keys() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(Clean)).unwrap();

    let req = DiagnosticsRequest::default();
    let limits = BundleLimits::default();
    let built = build_support_bundle_tar_zst(&r, &req, &limits).unwrap();

    let v: serde_json::Value = serde_json::from_slice(&built.manifest_json).unwrap();
    assert!(v.get("bundle_id").is_some());
    assert!(built.archive_bytes.len() > 100);

    let entries = read_tar_zst_entries(&built.archive_bytes).unwrap();
    let redacted = entries
        .iter()
        .find(|(p, _)| p == "meta/event.json")
        .expect("meta/event.json exists");
    let payload = String::from_utf8_lossy(&redacted.1);
    assert!(payload.contains("[REDACTED]"));
    assert!(!payload.contains("abcd1234EFGH5678IJKL9012MNOP3456"));
}
