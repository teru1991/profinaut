use serde_json::Value;

use ucel_testkit::golden::{assert_json_eq, repo_root_from_manifest_dir, GoldenWsFixture};

#[test]
fn golden_ws_bybit_normalize_orderbook_snapshot() {
    let repo_root = repo_root_from_manifest_dir();

    // fixture load
    let fx = GoldenWsFixture::load(&repo_root, "bybit", "orderbook_snapshot")
        .expect("load golden fixture");

    // normalize (Bybit already has normalize_ws_event)
    let endpoint_id = "crypto.public.ws.orderbook.l2";
    let evt = ucel_cex_bybit::normalize_ws_event(endpoint_id, &fx.raw).expect("normalize_ws_event");

    let actual: Value = serde_json::to_value(evt).expect("to_value");
    assert_json_eq(
        &actual,
        &fx.expected,
        "bybit: orderbook snapshot normalization must be stable",
    );
}
