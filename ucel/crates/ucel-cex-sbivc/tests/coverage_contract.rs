use std::collections::BTreeSet;

fn read_catalog_ids() -> BTreeSet<String> {
    let raw = include_str!("../../../../docs/exchanges/sbivc/catalog.json");
    let v: serde_json::Value = serde_json::from_str(raw).expect("valid catalog json");
    let mut set = BTreeSet::new();
    if let Some(arr) = v.get("rest_endpoints").and_then(|x| x.as_array()) {
        for it in arr {
            if let Some(id) = it.get("id").and_then(|x| x.as_str()) {
                set.insert(id.to_string());
            }
        }
    }
    if let Some(arr) = v.get("ws_channels").and_then(|x| x.as_array()) {
        for it in arr {
            if let Some(id) = it.get("id").and_then(|x| x.as_str()) {
                set.insert(id.to_string());
            }
        }
    }
    set
}

#[test]
fn coverage_ids_exist_in_catalog_and_are_public_market_data() {
    let catalog = read_catalog_ids();

    let cov = include_str!("../../../coverage/sbivc.yaml");
    let m: serde_yaml::Value = serde_yaml::from_str(cov).expect("valid coverage yaml");
    let entries = m
        .get("entries")
        .and_then(|x| x.as_sequence())
        .expect("entries");
    assert!(!entries.is_empty(), "coverage entries empty");

    for e in entries {
        let id = e.get("id").and_then(|x| x.as_str()).expect("entry id");
        assert!(
            catalog.contains(id),
            "coverage id not found in catalog: {id}"
        );
        assert!(
            id.contains(".public."),
            "non-public id in sbivc H coverage: {id}"
        );
        assert!(
            id.contains(".market_data."),
            "non-market_data id in sbivc H coverage: {id}"
        );
        assert_eq!(
            e.get("implemented").and_then(|x| x.as_bool()),
            Some(true),
            "implemented not true: {id}"
        );
        assert_eq!(
            e.get("tested").and_then(|x| x.as_bool()),
            Some(true),
            "tested not true: {id}"
        );
    }

    assert_eq!(
        m.get("strict").and_then(|x| x.as_bool()),
        Some(true),
        "sbivc strict must be true"
    );
}
