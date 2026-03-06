use std::collections::BTreeSet;

use ucel_registry::hub::Hub;

#[test]
fn hub_list_examples_smoke_shape() {
    let hub = Hub::default();
    let exchanges = hub.list_exchanges();
    assert!(exchanges.len() > 9, "expected >= 10 exchanges/families");

    let mut seen = BTreeSet::new();
    for exchange in exchanges {
        assert!(
            seen.insert(exchange.as_str().to_string()),
            "duplicate canonical id"
        );
        let (_rest, _ws) = hub
            .list_catalog_entries(exchange)
            .expect("catalog count should resolve");
    }
}
