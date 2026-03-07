#[derive(Debug, Clone, Copy)]
pub struct IrUnpackPolicy {
    pub max_entries: usize,
    pub max_total_bytes: u64,
    pub allow_nested_archive: bool,
}

impl Default for IrUnpackPolicy {
    fn default() -> Self {
        Self { max_entries: 32, max_total_bytes: 2 * 1024 * 1024, allow_nested_archive: false }
    }
}
