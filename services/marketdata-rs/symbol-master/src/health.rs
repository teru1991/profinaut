use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Default)]
pub struct Health {
    reasons: BTreeSet<String>,
    down: bool,
}

impl Health {
    pub fn mark_reason(&mut self, reason: impl Into<String>) {
        self.reasons.insert(reason.into());
    }

    pub fn clear_reason(&mut self, reason: &str) {
        self.reasons.remove(reason);
    }

    pub fn mark_down(&mut self) {
        self.down = true;
    }

    pub fn state(&self) -> HealthState {
        if self.down {
            HealthState::Down
        } else if self.reasons.is_empty() {
            HealthState::Healthy
        } else {
            HealthState::Degraded
        }
    }
}
