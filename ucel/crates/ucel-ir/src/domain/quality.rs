use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityStatus {
    Ok,
    Partial,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    pub status: QualityStatus,
    pub missing: Vec<String>,
    pub anomaly_flags: Vec<String>,
    pub http_status: Option<u16>,
    pub confidence: f32,
}

impl Default for Quality {
    fn default() -> Self {
        Self {
            status: QualityStatus::Ok,
            missing: Vec::new(),
            anomaly_flags: Vec::new(),
            http_status: None,
            confidence: 1.0,
        }
    }
}
