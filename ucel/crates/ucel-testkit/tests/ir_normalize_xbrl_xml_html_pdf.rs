use ucel_ir::normalize::{html::html_to_text, pdf::pdf_text_layer, xbrl::xbrl_to_text};
use ucel_testkit::ir_normalize::{build_pdf_bytes_from_text_fixture, fixture_path, load_text_fixture};

#[test]
fn structured_normalizers_preserve_content() {
    let html = load_text_fixture(&fixture_path("html/minimal_notice.html"));
    assert!(!html_to_text(&html).contains("bad()"));

    let x = load_text_fixture(&fixture_path("xbrl/minimal_fact_instance.xml"));
    assert!(xbrl_to_text(&x).contains("100"));

    let pdf = build_pdf_bytes_from_text_fixture(&fixture_path("pdf/minimal_text_layer.pdf.txt"));
    assert!(pdf_text_layer(&pdf).unwrap().contains("PDF IR NOTICE"));
}
