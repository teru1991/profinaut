use crate::health::Health;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsSignal {
    Lagged,
    Reconnected,
}

pub fn handle_ws_signal(health: &mut Health, signal: WsSignal) {
    match signal {
        WsSignal::Lagged => health.mark_reason("ws_failed"),
        WsSignal::Reconnected => health.clear_reason("ws_failed"),
    }
}
