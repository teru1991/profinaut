use std::path::Path;

use ucel_cex_bitbank::private::request_builders::build_get_assets_request;
use ucel_core::PrivateRestOperation;
use ucel_testkit::private_rest::{load_expected_canonical, normalize_reason};
use ucel_transport::redaction::{redact_headers, RedactionPolicy};

#[test]
fn request_shape_and_redaction_for_representative_venue() {
    let req = build_get_assets_request("real-key", "100", "real-signature");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/v1/user/assets");

    let redacted = redact_headers(&RedactionPolicy::default(), &req.headers);
    let joined = format!("{:?}", redacted);
    assert!(!joined.contains("real-key"));
    assert!(!joined.contains("real-signature"));
}

#[test]
fn fixture_raw_to_canonical_expectation_is_present() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..");
    let canonical = load_expected_canonical(&root, "bitbank").expect("fixture should parse");
    assert!(!canonical.balances.is_empty());
    assert_eq!(canonical.balances[0].asset, "BTC");
}

#[test]
fn cancel_get_order_fills_reason_mapping_is_available() {
    let (cancel_class, _) = normalize_reason(403, "forbidden", PrivateRestOperation::CancelOrder);
    let (order_class, _) = normalize_reason(404, "not found", PrivateRestOperation::GetOrder);
    let (fills_class, _) = normalize_reason(429, "rate limit", PrivateRestOperation::GetFills);

    assert_eq!(format!("{:?}", cancel_class), "Forbidden");
    assert_eq!(format!("{:?}", order_class), "NotFound");
    assert_eq!(format!("{:?}", fills_class), "RateLimited");
}
