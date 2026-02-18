//! Persistence layer for the crypto-collector framework v1.4.
//!
//! These types are intentionally not wired into the binary entry-point yet;
//! they will be connected in Tasks E and F.
#![allow(dead_code)]
//!
//! ## Modules
//!
//! | Module       | Purpose                                                |
//! |--------------|--------------------------------------------------------|
//! | `envelope`   | Envelope v1 — canonical data type for raw messages     |
//! | `sink`       | `Sink` trait + `SinkState` + `SinkError`               |
//! | `metrics`    | `PersistenceMetrics` — atomic counters and gauges      |
//! | `mongo`      | D1: `MongoSink` with `insert_many`, retry, state       |
//! | `spool`      | D2: `DurableSpool` — append-only, crash-safe segments  |
//! | `replay`     | D3: `ReplayWorker` — background spool-to-Mongo drain   |
//! | `dedup`      | D4: `DedupWindow` — bounded time-windowed dedup        |
//! | `pipeline`   | D5: `PipelineSink` — full fallback integration         |
//!
//! ## Fallback chain (implemented in `pipeline`)
//!
//! ```text
//! write_batch(batch)
//!   → [dedup filter]
//!   → try Mongo (D1)
//!       → Ok: done
//!       → Err(MongoUnavailable): try Spool (D2)
//!           → Ok: queued for replay (D3)
//!           → Spool full: on_full policy
//!               drop_all / drop_ticker_depth_keep_trade → Ok (metric++)
//!               block → wait + retry
//! ```

pub mod dedup;
pub mod envelope;
pub mod metrics;
pub mod mongo;
pub mod pipeline;
pub mod replay;
pub mod sink;
pub mod spool;

// Convenience re-exports for the most commonly used types.
#[allow(unused_imports)]
pub use envelope::Envelope;
#[allow(unused_imports)]
pub use metrics::PersistenceMetrics;
#[allow(unused_imports)]
pub use pipeline::{PipelineConfig, PipelineSink};
#[allow(unused_imports)]
pub use sink::{Sink, SinkError, SinkState};
