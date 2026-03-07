use ucel_ir::normalize::{safety::IrUnpackPolicy, zip::unpack_zip};
use ucel_testkit::ir_normalize::{build_zip_bytes_from_spec, fixture_path};

#[test]
fn zip_policy_guards_work() {
    let ok = build_zip_bytes_from_spec(&fixture_path("zip/minimal_archive_spec.json"));
    let attachments = unpack_zip(&ok, IrUnpackPolicy::default()).expect("must unpack");
    assert!(!attachments.is_empty());

    let traversal = build_zip_bytes_from_spec(&fixture_path("zip/path_traversal_archive_spec.json"));
    assert!(unpack_zip(&traversal, IrUnpackPolicy::default()).is_err());
}
