#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureAction {
    Throttle,
    Stop,
}

pub fn decide_backpressure_action(journal_ok: bool, queue_saturated: bool) -> BackpressureAction {
    if !journal_ok || queue_saturated { BackpressureAction::Stop } else { BackpressureAction::Throttle }
}
