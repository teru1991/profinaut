use ucel_core::{IrArtifactDescriptor, IrArtifactKind, IrNormalizedFormat};

pub fn detect_format(meta: &IrArtifactDescriptor, bytes: &[u8]) -> Option<IrNormalizedFormat> {
    if bytes.starts_with(b"%PDF-") { return Some(IrNormalizedFormat::Pdf); }
    if bytes.starts_with(b"PK\x03\x04") { return Some(IrNormalizedFormat::Zip); }
    if let Some(ct) = &meta.content_type {
        let ct = ct.to_ascii_lowercase();
        if ct.contains("html") { return Some(IrNormalizedFormat::Html); }
        if ct.contains("xml") {
            let body = String::from_utf8_lossy(bytes).to_ascii_lowercase();
            if body.contains("xbrli:xbrl") { return Some(IrNormalizedFormat::Xbrl); }
            if body.contains("ix:") { return Some(IrNormalizedFormat::Ixbrl); }
            if body.contains("<rss") { return Some(IrNormalizedFormat::Rss); }
            return Some(IrNormalizedFormat::Xml);
        }
        if ct.contains("json") { return Some(IrNormalizedFormat::Json); }
        if ct.contains("csv") { return Some(IrNormalizedFormat::Csv); }
        if ct.contains("plain") { return Some(IrNormalizedFormat::Txt); }
    }
    match meta.kind {
        IrArtifactKind::Html => Some(IrNormalizedFormat::Html),
        IrArtifactKind::Pdf => Some(IrNormalizedFormat::Pdf),
        IrArtifactKind::Xbrl => Some(IrNormalizedFormat::Xbrl),
        IrArtifactKind::Ixbrl => Some(IrNormalizedFormat::Ixbrl),
        IrArtifactKind::Xml => Some(IrNormalizedFormat::Xml),
        IrArtifactKind::Txt => Some(IrNormalizedFormat::Txt),
        IrArtifactKind::Csv => Some(IrNormalizedFormat::Csv),
        IrArtifactKind::Zip => Some(IrNormalizedFormat::Zip),
        IrArtifactKind::Json => Some(IrNormalizedFormat::Json),
        IrArtifactKind::Rss => Some(IrNormalizedFormat::Rss),
    }
}
