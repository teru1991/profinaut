fn main() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bundle_path = root.join("fixtures/support_bundle/bundle_v1.json");
    let bundle: serde_json::Value = serde_json::from_slice(&std::fs::read(bundle_path).expect("read fixture")).expect("parse fixture");
    let (summary, compat, drift) = ucel_diagnostics_analyzer::analyze_support_bundle_value(&bundle, &root).expect("analyze");
    println!("summary={}", serde_json::to_string_pretty(&summary).unwrap());
    println!("compat={}", serde_json::to_string_pretty(&compat).unwrap());
    println!("drift={}", serde_json::to_string_pretty(&drift).unwrap());
}
