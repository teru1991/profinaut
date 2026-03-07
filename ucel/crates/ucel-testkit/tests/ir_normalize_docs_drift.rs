use ucel_testkit::ir_normalize::load_text_fixture;

#[test]
fn docs_include_binary_free_policy() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ir/ir_content_normalization.md");
    let txt = load_text_fixture(root.to_str().unwrap());
    assert!(txt.contains("binary-free fixture policy"));
    assert!(txt.contains("html") && txt.contains("zip"));
}
