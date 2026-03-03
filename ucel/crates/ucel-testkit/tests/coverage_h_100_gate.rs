use std::path::Path;

#[test]
fn strict_ws_venues_have_v2_coverage_and_golden_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let strict = ucel_testkit::coverage_v2::load_strict_venues(&root).expect("load strict venues");

    assert!(
        !strict.strict_ws_golden.contains(&"sbivc".to_string()),
        "sbivc must not be part of strict ws venues"
    );

    for venue in strict.strict_ws_golden {
        let p = root
            .join("ucel/coverage/coverage_v2/exchanges")
            .join(format!("{venue}.json"));
        assert!(p.exists(), "missing coverage_v2 file: {}", p.display());
        let v = ucel_testkit::coverage_v2::load_json(&p).expect("parse coverage_v2 json");
        assert!(
            ucel_testkit::coverage_v2::public_rest(&v).expect("public.rest"),
            "public.rest must be true for strict venue={venue}"
        );

        if ucel_testkit::coverage_v2::public_ws(&v).expect("public.ws") {
            let fx_raw = root
                .join("ucel/fixtures/golden/ws")
                .join(&venue)
                .join("raw.json");
            let fx_exp = root
                .join("ucel/fixtures/golden/ws")
                .join(&venue)
                .join("expected.normalized.json");
            assert!(fx_raw.exists(), "missing fixture raw: {}", fx_raw.display());
            assert!(
                fx_exp.exists(),
                "missing fixture expected: {}",
                fx_exp.display()
            );
        }
    }
}
