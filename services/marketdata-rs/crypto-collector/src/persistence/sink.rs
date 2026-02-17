//! Sink trait, SinkState, and SinkError — stable Task C/D/E/F interface.

use async_trait::async_trait;
use thiserror::Error;

use super::envelope::Envelope;

// ---------------------------------------------------------------------------
// SinkState
// ---------------------------------------------------------------------------

/// Observable state of a `Sink` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkState {
    /// Last write succeeded.
    Ok,
    /// Underlying store is currently unreachable (transient).
    MongoUnavailable,
    /// Repeated consecutive failures; data may be at risk.
    Degraded,
}

// ---------------------------------------------------------------------------
// SinkError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SinkError {
    #[error("mongo unavailable after {retries} retr{}: {msg}", if *retries == 1 { "y" } else { "ies" })]
    MongoUnavailable { retries: u32, msg: String },

    #[error("spool full (on_full policy: {policy})")]
    SpoolFull { policy: String },

    #[error("spool I/O error: {0}")]
    SpoolIo(#[from] std::io::Error),

    #[error("serialisation error: {0}")]
    Serialise(String),

    #[error("{0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// Sink trait
// ---------------------------------------------------------------------------

/// Asynchronous batch sink for `Envelope` values.
///
/// Implementations must be `Send + Sync` so they can be shared across tasks.
/// `state()` returns the current health of the underlying store and must be
/// cheap to call (no I/O).
#[async_trait]
pub trait Sink: Send + Sync {
    /// Write a batch of envelopes.  Returns `Ok(())` if every envelope in
    /// `batch` was durably persisted (or accepted by the spool for later
    /// replay).  Returns `Err` only if persistence failed unrecoverably for
    /// this call.
    async fn write_batch(&self, batch: Vec<Envelope>) -> Result<(), SinkError>;

    /// Current observable state of the sink (no I/O).
    fn state(&self) -> SinkState;
}
