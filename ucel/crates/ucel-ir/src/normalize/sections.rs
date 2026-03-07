use ucel_core::{IrNormalizationProvenance, IrNormalizedSection};

pub fn sections_from_text(text: &str) -> Vec<IrNormalizedSection> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    for (i, line) in text.lines().enumerate() {
        if line.starts_with('#') || line.starts_with("第") || line.to_ascii_lowercase().starts_with("section") {
            out.push(IrNormalizedSection {
                heading_level: Some(1),
                title: line.trim_matches('#').trim().to_string(),
                ordinal: out.len(),
                text_range: (idx, idx + line.len()),
                provenance: IrNormalizationProvenance { source_type: Some("text_heading".into()), source_ref: Some(format!("line:{}", i + 1)), context_ref: None, extra: Default::default() },
            });
        }
        idx += line.len() + 1;
    }
    if out.is_empty() {
        out.push(IrNormalizedSection {
            heading_level: None,
            title: "body".into(),
            ordinal: 0,
            text_range: (0, text.len()),
            provenance: IrNormalizationProvenance::default(),
        });
    }
    out
}
