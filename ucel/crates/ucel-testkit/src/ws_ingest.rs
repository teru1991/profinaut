use ucel_core::{
    escalate_integrity_failure, IngestFailureClass, IngestIntegrityMode, IngestLifecycleState,
    IngestRetryBudget, IngestStreamKey,
};
use ucel_transport::ws::supervisor::WsIngestSupervisor;

pub fn sample_key(scope: &str) -> IngestStreamKey {
    IngestStreamKey {
        exchange: "binance".into(),
        family: "spot".into(),
        channel: "trades".into(),
        symbol: "BTCUSDT".into(),
        shard: 0,
        auth_scope: scope.into(),
    }
}

pub fn sample_budget() -> IngestRetryBudget {
    IngestRetryBudget {
        max_retries: 3,
        max_checksum_retries: 1,
        max_heartbeat_retries: 2,
    }
}

pub fn can_escalate_checksum(retries: u32) -> bool {
    escalate_integrity_failure(
        retries,
        sample_budget(),
        IngestFailureClass::ChecksumMismatch,
    )
}

pub fn make_supervisor_active(scope: &str) -> WsIngestSupervisor {
    let mut sup = WsIngestSupervisor::default();
    sup.transition(
        sample_key(scope),
        IngestLifecycleState::Planned,
        IngestLifecycleState::Active,
        None,
        None,
        "boot",
    );
    sup
}

pub fn mismatch_directive() -> ucel_core::IngestResumeDirective {
    ucel_core::failure_to_resume_directive(
        IngestFailureClass::ChecksumMismatch,
        IngestIntegrityMode::SequenceAndChecksum,
    )
}
