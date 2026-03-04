use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct BundleLimits {
    pub max_total_bytes: u64,
    pub max_files: usize,
    pub max_single_file_bytes: u64,
    pub max_path_len: usize,
    pub max_concurrent_builds: usize,
    pub max_build_time: Duration,
}

impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 32 * 1024 * 1024,
            max_files: 2048,
            max_single_file_bytes: 4 * 1024 * 1024,
            max_path_len: 256,
            max_concurrent_builds: 2,
            max_build_time: Duration::from_secs(30),
        }
    }
}

static ACTIVE_BUILDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, thiserror::Error)]
pub enum BundleBuildError {
    #[error("bundle build rejected: too many concurrent builds")]
    TooManyConcurrentBuilds,
    #[error("bundle build rejected: too many files: {0}")]
    TooManyFiles(usize),
    #[error("bundle build rejected: total size exceeded: {0} bytes")]
    TotalSizeExceeded(u64),
    #[error("bundle build rejected: single file too large: {path} ({size} bytes)")]
    SingleFileTooLarge { path: String, size: u64 },
    #[error("bundle build rejected: invalid path: {0}")]
    InvalidPath(String),
    #[error("bundle build rejected: path too long: {0}")]
    PathTooLong(String),
    #[error("bundle build failed: io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bundle build failed: serialize: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("bundle build failed: time limit exceeded")]
    TimeLimitExceeded,
}

pub struct BuildGuard;

impl BuildGuard {
    pub fn try_acquire(limits: &BundleLimits) -> Result<Self, BundleBuildError> {
        loop {
            let cur = ACTIVE_BUILDS.load(Ordering::Acquire);
            if cur >= limits.max_concurrent_builds {
                return Err(BundleBuildError::TooManyConcurrentBuilds);
            }
            if ACTIVE_BUILDS
                .compare_exchange(cur, cur + 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(Self);
            }
        }
    }
}

impl Drop for BuildGuard {
    fn drop(&mut self) {
        ACTIVE_BUILDS.fetch_sub(1, Ordering::AcqRel);
    }
}
