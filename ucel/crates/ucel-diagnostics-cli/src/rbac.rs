use std::time::{Duration, SystemTime};

#[derive(Clone, Debug)]
pub struct BreakGlass {
    pub ttl: Duration,
    pub reason: String,
    pub approvals: Vec<String>,
    #[allow(dead_code)]
    pub approved_at: SystemTime,
}

#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error("invalid ttl")]
    InvalidTtl,
    #[error("missing reason")]
    MissingReason,
    #[error("insufficient approvals")]
    InsufficientApprovals,
}

pub fn check_break_glass(
    ttl_minutes: u32,
    reason: &str,
    approvals: &[String],
) -> Result<BreakGlass, RbacError> {
    if ttl_minutes == 0 {
        return Err(RbacError::InvalidTtl);
    }
    if reason.trim().is_empty() {
        return Err(RbacError::MissingReason);
    }
    let required = std::env::var("UCEL_DIAG_BG_APPROVALS_REQUIRED")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2);
    if approvals.len() < required {
        return Err(RbacError::InsufficientApprovals);
    }

    Ok(BreakGlass {
        ttl: Duration::from_secs(ttl_minutes as u64 * 60),
        reason: reason.to_string(),
        approvals: approvals.to_vec(),
        approved_at: SystemTime::now(),
    })
}

pub fn current_actor() -> String {
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".to_string());
    format!("{user}@{host}")
}
