use ucel_core::{IrNormalizationProvenance, IrNormalizedTable};

pub fn csv_to_table(text: &str) -> IrNormalizedTable {
    let mut lines = text.lines();
    let headers = lines.next().unwrap_or_default().split(',').map(|s| s.trim().to_string()).collect();
    let rows = lines.map(|l| l.split(',').map(|s| s.trim().to_string()).collect()).collect();
    IrNormalizedTable {
        caption: Some("csv".into()),
        headers,
        rows,
        provenance: IrNormalizationProvenance { source_type: Some("csv".into()), source_ref: None, context_ref: None, extra: Default::default() },
    }
}
