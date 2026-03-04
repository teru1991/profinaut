use std::{env, fs, path::PathBuf};
use ucel_diagnostics_analyzer::analyze_tar_zst_bundle;

#[test]
fn compares_external_golden_bundle_when_env_is_set() {
    let Some(dir) = env::var_os("UCEL_GOLDEN_BUNDLE_DIR") else {
        return;
    };

    let base = PathBuf::from(dir);
    let bundle = base.join("bundle_minimal.tar.zst");
    let expected = base.join("expected.summary.json");

    assert!(
        bundle.exists(),
        "missing external golden bundle: {}",
        bundle.display()
    );
    assert!(
        expected.exists(),
        "missing expected summary fixture: {}",
        expected.display()
    );

    let (_, summary) = analyze_tar_zst_bundle(fs::read(&bundle).expect("read bundle bytes"))
        .expect("analyze external bundle");

    let mut actual = serde_json::to_value(summary).expect("serialize summary");
    strip_dynamic_fields(&mut actual);

    let mut expected_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&expected).expect("read expected json"))
            .expect("parse expected json");
    strip_dynamic_fields(&mut expected_json);

    assert_eq!(actual, expected_json);
}

fn strip_dynamic_fields(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        obj.remove("bundle_id");
        obj.remove("created_at_rfc3339");
        obj.remove("total_bytes");
    }
}
