use ucel_registry::hub::Hub;
use ucel_sdk::config::SdkConfig;
use ucel_sdk::support_bundle::generate_support_bundle;

#[test]
fn support_bundle_redacts_secrets_and_has_required_shape() {
    let cfg = SdkConfig::default();
    let hub = Hub::default();
    let transport_diag = serde_json::json!({
        "state": "connected",
        "sample": "ok"
    });

    let bundle = generate_support_bundle(&cfg, &hub, transport_diag);
    let as_text = serde_json::to_string(&bundle).unwrap();

    for banned in [
        "api_key",
        "api_secret",
        "Authorization",
        "Cookie",
        "signature",
    ] {
        assert!(!as_text.contains(banned), "banned token leaked: {banned}");
    }

    for key in ["metadata", "ssot", "transport", "hub", "wal", "errors"] {
        assert!(bundle.get(key).is_some(), "missing key: {key}");
    }
}
