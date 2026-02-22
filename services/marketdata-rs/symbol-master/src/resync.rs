use ucel_symbol_core::Snapshot;

#[derive(Debug, Clone, Default)]
pub struct ResyncController {
    pub stale: bool,
}

impl ResyncController {
    pub fn on_restore(&mut self) {
        self.stale = true;
    }

    pub fn on_fresh_snapshot(&mut self, _snapshot: &Snapshot) {
        self.stale = false;
    }
}
