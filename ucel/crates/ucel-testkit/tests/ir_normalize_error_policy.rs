use ucel_ir::normalize::{charset::normalize_to_utf8, pdf::pdf_text_layer};

#[test]
fn fail_fast_for_invalid_charset_and_pdf() {
    assert!(normalize_to_utf8(&[0xff, 0xfe], None).is_err());
    assert!(pdf_text_layer(b"NOTPDF").is_err());
}
