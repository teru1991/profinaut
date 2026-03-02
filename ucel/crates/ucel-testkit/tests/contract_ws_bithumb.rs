use serde_json::Value;

use ucel_testkit::golden::{assert_json_eq, repo_root_from_manifest_dir, GoldenWsFixture};

#[test]
fn golden_ws_bithumb_normalize_trade_snapshot() {
    let repo_root = repo_root_from_manifest_dir();
    let fx = GoldenWsFixture::load(&repo_root, "bithumb", "trade_snapshot").expect("load fixture");

    let evt = ucel_cex_bithumb::normalize_ws_event("openapi.public.ws.trade.snapshot", &fx.raw)
        .expect("normalize ws event");

    let actual: Value = serde_json::to_value(evt).expect("to_value");
    assert_json_eq(
        &actual,
        &fx.expected,
        "bithumb: trade snapshot normalization must be stable",
    );
}
