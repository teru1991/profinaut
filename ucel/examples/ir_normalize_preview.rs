use ucel_ir::normalize::detect::detect_format;
use ucel_core::{IrArtifactDescriptor, IrArtifactKey, IrArtifactKind, IrArtifactSource, IrDocumentKey};

fn main() {
    let meta = IrArtifactDescriptor {
        key: IrArtifactKey { document: IrDocumentKey { source_id: "demo".into(), source_document_id: "doc".into() }, artifact_id: "a1".into() },
        source_id: "demo".into(),
        kind: IrArtifactKind::Pdf,
        content_type: Some("application/pdf".into()),
        source: IrArtifactSource::ByteSource,
        checksum_sha256: None,
        size_bytes: None,
        encoding: Some("utf-8".into()),
    };
    println!("detected={:?}", detect_format(&meta, b"%PDF-1.4"));
}
