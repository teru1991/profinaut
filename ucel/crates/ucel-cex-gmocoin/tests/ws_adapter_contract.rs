use serde_json::json;

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;
use ucel_transport::ws::adapter::{InboundClass, WsVenueAdapter};

#[test]
fn build_subscribe_matches_coverage_op_ids() {
    let a = GmoCoinWsAdapter::new();

    let msg = a
        .build_subscribe("crypto.public.ws.ticker.update", "BTC/JPY", &json!({}))
        .unwrap();
    assert_eq!(msg.len(), 1);
    assert!(msg[0].text.contains("\"channel\":\"ticker\""));
    assert!(msg[0].text.contains("\"symbol\":\"BTC_JPY\""));

    let msg = a
        .build_subscribe("crypto.public.ws.trades.update", "BTC/JPY", &json!({}))
        .unwrap();
    assert!(msg[0].text.contains("\"channel\":\"trades\""));

    let msg = a
        .build_subscribe("crypto.public.ws.orderbooks.update", "BTC/JPY", &json!({}))
        .unwrap();
    assert!(msg[0].text.contains("\"channel\":\"orderbooks\""));
}

#[test]
fn build_subscribe_rejects_unknown_op_id() {
    let a = GmoCoinWsAdapter::new();
    let err = a
        .build_subscribe("crypto.public.ws.unknown.update", "BTC/JPY", &json!({}))
        .err()
        .unwrap();
    assert!(err.contains("unsupported op_id"));
}

#[test]
fn classify_inbound_maps_channel_to_op_id_and_symbol() {
    let a = GmoCoinWsAdapter::new();

    let raw = json!({
        "channel": "ticker",
        "symbol": "BTC_JPY"
    })
    .to_string();

    match a.classify_inbound(raw.as_bytes()) {
        InboundClass::Data {
            op_id,
            symbol,
            params_canon_hint,
        } => {
            assert_eq!(op_id.unwrap(), "crypto.public.ws.ticker.update");
            assert_eq!(symbol.unwrap(), "BTC/JPY");
            assert_eq!(params_canon_hint.unwrap(), "{}");
        }
        other => panic!("unexpected inbound class: {other:?}"),
    }
}
