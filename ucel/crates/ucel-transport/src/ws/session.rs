use ucel_core::{PrivateWsLifecycleState, PrivateWsRejectClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateSessionMeta {
    pub session_id: String,
    pub venue: String,
    pub state: PrivateWsLifecycleState,
    pub last_reject: Option<PrivateWsRejectClass>,
}

impl PrivateSessionMeta {
    pub fn transition(&mut self, next: PrivateWsLifecycleState) -> bool {
        if self.state.can_transition_to(next) {
            self.state = next;
            true
        } else {
            false
        }
    }
}
