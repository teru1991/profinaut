use ucel_core::{IrArtifactDescriptor, IrArtifactKey, IrArtifactKind, IrArtifactSource, IrDocumentKey};
use ucel_ir::normalize::detect::detect_format;

fn meta(kind: IrArtifactKind, content_type: Option<&str>) -> IrArtifactDescriptor {
    IrArtifactDescriptor { key: IrArtifactKey { document: IrDocumentKey { source_id: "s".into(), source_document_id: "d".into() }, artifact_id: "a".into() }, source_id: "s".into(), kind, content_type: content_type.map(str::to_string), source: IrArtifactSource::ByteSource, checksum_sha256: None, size_bytes: None, encoding: Some("utf-8".into()) }
}

#[test]
fn detect_pdf_and_zip_magic() {
    assert_eq!(detect_format(&meta(IrArtifactKind::Txt, None), b"%PDF-1.4 x"), Some(ucel_core::IrNormalizedFormat::Pdf));
    assert_eq!(detect_format(&meta(IrArtifactKind::Txt, None), b"PK\x03\x04x"), Some(ucel_core::IrNormalizedFormat::Zip));
}

#[test]
fn detect_from_content_type_and_kind() {
    assert_eq!(detect_format(&meta(IrArtifactKind::Html, Some("text/html")), b"<html></html>"), Some(ucel_core::IrNormalizedFormat::Html));
    assert_eq!(detect_format(&meta(IrArtifactKind::Csv, Some("text/plain")), b"a,b"), Some(ucel_core::IrNormalizedFormat::Txt));
}
