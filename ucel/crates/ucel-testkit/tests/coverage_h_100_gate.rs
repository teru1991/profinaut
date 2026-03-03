use std::path::Path;

#[test]
fn h_coverage_strict_deribit_sbivc_is_100_percent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for venue in ["deribit", "sbivc"] {
        let p = root.join("ucel/coverage").join(format!("{venue}.yaml"));
        assert!(p.exists(), "missing coverage file: {}", p.display());
        let raw = std::fs::read_to_string(&p).expect("read coverage");
        let m: serde_yaml::Value = serde_yaml::from_str(&raw).expect("parse coverage");
        assert_eq!(
            m.get("strict").and_then(|x| x.as_bool()),
            Some(true),
            "venue strict must be true: {venue}"
        );
        let entries = m
            .get("entries")
            .and_then(|x| x.as_sequence())
            .expect("entries");
        assert!(!entries.is_empty(), "entries empty for venue={venue}");
        for e in entries {
            let id = e.get("id").and_then(|x| x.as_str()).unwrap_or("<id>");
            assert_eq!(
                e.get("implemented").and_then(|x| x.as_bool()),
                Some(true),
                "implemented not true: {venue} {id}"
            );
            assert_eq!(
                e.get("tested").and_then(|x| x.as_bool()),
                Some(true),
                "tested not true: {venue} {id}"
            );
        }
        let fx_raw = root
            .join("ucel/fixtures/golden/ws")
            .join(venue)
            .join("raw.json");
        let fx_exp = root
            .join("ucel/fixtures/golden/ws")
            .join(venue)
            .join("expected.normalized.json");
        assert!(fx_raw.exists(), "missing fixture raw: {}", fx_raw.display());
        assert!(
            fx_exp.exists(),
            "missing fixture expected: {}",
            fx_exp.display()
        );
    }
}
