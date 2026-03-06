use ucel_diagnostics_core::provider::*;
use ucel_diagnostics_core::registry::*;

struct P1;
impl DiagnosticsProvider for P1 {
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
                size_limit_bytes: 128,
                content: ContributionContent::Text("hello".into()),
            },
            Contribution {
                provider_id: "alpha".into(),
                kind: ContributionKind::Json,
                path: "meta/info.json".into(),
                mime: "application/json".into(),
                size_limit_bytes: 128,
                content: ContributionContent::Json(serde_json::json!({"k":"v"})),
            },
        ]
    }
}

struct P2;
impl DiagnosticsProvider for P2 {
    fn meta(&self) -> ProviderMeta {
        ProviderMeta {
            provider_id: "beta".into(),
            provider_version: "0.1.0".into(),
        }
    }
    fn contributions(&self, _req: &DiagnosticsRequest) -> Vec<Contribution> {
        vec![Contribution {
            provider_id: "beta".into(),
            kind: ContributionKind::Meta,
            path: "meta/z.txt".into(),
            mime: "".into(),
            size_limit_bytes: 128,
            content: ContributionContent::Text("z".into()),
        }]
    }
}

#[test]
fn registry_orders_deterministically() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(P2)).unwrap();
    r.register(Box::new(P1)).unwrap();

    let req = DiagnosticsRequest::default();
    let cs = r.collect(&req).unwrap();
    let keys: Vec<(String, String, String)> = cs
        .iter()
        .map(|c| {
            let k = match c.kind {
                ContributionKind::Json => "json",
                ContributionKind::Text => "text",
                ContributionKind::Meta => "meta",
                ContributionKind::Binary => "binary",
            };
            (c.provider_id.clone(), c.path.clone(), k.to_string())
        })
        .collect();

    assert_eq!(
        keys,
        vec![
            ("alpha".into(), "logs/tail.txt".into(), "text".into()),
            ("alpha".into(), "meta/info.json".into(), "json".into()),
            ("beta".into(), "meta/z.txt".into(), "meta".into()),
        ]
    );
}

#[test]
fn registry_rejects_traversal_paths() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());

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
                path: "../secret.txt".into(),
                mime: "text/plain".into(),
                size_limit_bytes: 10,
                content: ContributionContent::Text("no".into()),
            }]
        }
    }

    r.register(Box::new(Bad)).unwrap();
    let err = r.collect(&DiagnosticsRequest::default()).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("invalid contribution path"));
}

#[test]
fn registry_rejects_duplicate_provider_id() {
    let mut r = DiagnosticsRegistry::new(RegistryLimits::default());
    r.register(Box::new(P1)).unwrap();
    let err = r.register(Box::new(P1)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("duplicate provider_id"));
}
