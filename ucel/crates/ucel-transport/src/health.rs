use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReason {
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportHealth {
    pub status: HealthStatus,
    pub reasons: Vec<HealthReason>,
}

impl TransportHealth {
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            reasons: vec![],
        }
    }

    pub fn degraded(reasons: Vec<HealthReason>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            reasons,
        }
    }

    pub fn unhealthy(reasons: Vec<HealthReason>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            reasons,
        }
    }

    pub fn unknown(reason: HealthReason) -> Self {
        Self {
            status: HealthStatus::Unknown,
            reasons: vec![reason],
        }
    }
}
