# Multi-Exchange Market Data Collector Framework v1.4

## Overview

The framework provides a multi-exchange crypto market-data collection pipeline
implemented in Rust (`services/marketdata-rs/crypto-collector/`).

## Task Map

| Task | Scope | Status |
|------|-------|--------|
| A | Service skeleton, config, descriptor, health endpoint | Done |
| B | DSL execution, WS/REST runtime | Pending |
| C | Ingestion batcher | Pending (interfaces defined in Task D) |
| **D** | **Persistence: Mongo sink + spool + replay + dedup** | **Done** |
| E | WS/REST runtime integration | Pending |
| F | Metrics export, observability | Pending |

---

## Persistence Layer (Task D)

### Envelope v1

All messages are wrapped in an `Envelope` before persistence:

```rust
pub struct Envelope {
    pub message_id:     Option<String>,   // exchange-provided ID
    pub sequence:       Option<u64>,      // monotonic sequence
    pub exchange:       String,           // "binance", "kraken", …
    pub channel:        String,           // "trades", "orderbook", …
    pub symbol:         String,           // "BTC/USDT", …
    pub server_time_ms: Option<i64>,      // exchange timestamp (ms)
    pub received_at_ms: i64,              // local receive timestamp (ms)
    pub payload:        serde_json::Value, // raw JSON
}
```

### Sink Trait

```rust
#[async_trait]
pub trait Sink: Send + Sync {
    async fn write_batch(&self, batch: Vec<Envelope>) -> Result<(), SinkError>;
    fn state(&self) -> SinkState;
}

pub enum SinkState { Ok, MongoUnavailable, Degraded }
```

### Fallback Chain (PipelineSink)

```
write_batch(batch)
  1. [dedup.enabled] filter duplicates → dedup_dropped_total++
  2. try MongoSink.write_batch
       → Ok:  done
       → Err(MongoUnavailable):
           3. [spool.enabled] DurableSpool.append_batch
                → Ok: queued for replay
                → spool full: apply on_full policy
                     drop_all                    → drop, metric++
                     drop_ticker_depth_keep_trade → drop orderbook; block trades
                     block                       → wait until space
```

---

## Durability Semantics

### Mongo Sink (D1)

- Uses `insert_many` (bulk write) for efficiency.
- Bounded retry: `max_retries` attempts with exponential backoff
  (`retry_base_ms * 2^attempt` ms).
- State transitions:
  - `Ok` → on every successful batch
  - `MongoUnavailable` → after first exhausted-retry batch failure
  - `Degraded` → after `mongo_consecutive_failures_for_degraded` consecutive failures
- Metrics: `write_batch_latency_ms` (last batch), `ingest_errors_total{exchange}`

### Durable Spool (D2)

- Append-only, length-prefix-encoded segment files on disk.
- Segment file naming: `spool_{seq:06}.dat` (lexicographic = chronological).
- Record format: `[u32 LE: body length][JSON bytes of Envelope]`
- **Crash safety**: on startup the last (current write) segment is scanned;
  any partial (truncated) record at the end is removed.  All complete records
  are preserved.
- **Segment rotation**: new segment when `file_bytes > 0 && file_bytes + next_frame > max_segment_bytes`.
  A fresh segment always accepts the first record regardless of its size.
- **Total cap** (`max_total_mb`): `on_full` policy applied when cap is reached.
- Metrics: `spool_bytes` (gauge), `spool_segments` (gauge),
  `spool_dropped_total{exchange,channel}`, `spool_replay_total`

### Replay Worker (D3)

- Background Tokio task.
- Replays **oldest complete segments first** (ascending sequence number).
  The current write segment is never replayed (it may still be receiving writes).
- A segment is **deleted only after** a successful `insert_many`.
  Partial replays are safe: on restart the segment is retried from the beginning.
- Rate-limited: `replay_rate_limit_ms` sleep between successful batches.
- Shutdown-safe: monitors a `watch::Receiver<bool>` channel; exits cleanly.

### Dedup Window (D4)

- Optional (`dedup.enabled = false` by default).
- Bounded: `window_seconds` + `max_keys` (both enforced, no memory leak).
- Eviction: time-based (oldest-first) + hard cap (trim to `max_keys - 1` before
  each insertion).
- Dedup key priority:
  1. `mid:<message_id>` — if `message_id` is present
  2. `seq:<exchange>:<channel>:<sequence>` — if `sequence` is present
  3. `hash:<16 hex chars>` — DefaultHasher of serialised payload (fallback)
- Metrics: `dedup_dropped_total{exchange,channel}`

---

## Configuration

See `docs/config_reference_collector_toml.md` for the full TOML reference.

```toml
[persistence]
mongo_uri       = "mongodb://localhost:27017"
mongo_database  = "market_data"
mongo_collection = "crypto_envelopes"
mongo_max_retries = 3
mongo_retry_base_ms = 100
mongo_consecutive_failures_for_degraded = 3

[persistence.spool]
enabled        = false
dir            = "/var/spool/crypto-collector"
max_segment_mb = 64
max_total_mb   = 1024
on_full        = "drop_ticker_depth_keep_trade"

[persistence.dedup]
enabled        = false
window_seconds = 300
max_keys       = 100000
```

---

## Testing

```bash
cd services/marketdata-rs
cargo test -p crypto-collector
```

51 tests cover:
- Mongo sink retry, state transitions, degraded recovery
- Spool write/read, segment rotation, on_full policies, partial-write recovery
- Replay worker oldest-first, no-delete-on-failure, shutdown safety
- Dedup window time-eviction, max_keys cap, labeled metrics
- Pipeline integration: Mongo success path, fallback to spool, dedup filtering,
  fake integration (spool grows then drains, metrics tracked)

---

## Deployment Notes

- The `mongodb` driver crate is **not** compiled by default.  All persistence
  logic is fully unit-tested via `FakeMongoTarget`.  See
  `docs/troubleshooting.md` for instructions on wiring a real MongoDB instance.
- Spool and dedup are **disabled by default** (`enabled = false`).
- The `Sink` trait is the stable interface for Tasks E and F.

## Task F Final Integration (Crypto mock harness)

- `/healthz` now reports connector instance, per-instance and per-connection runtime state, persistence spool/dedup state, and time-quality summary for the mock crypto runtime.
- `/metrics` now exports Prometheus text payload for ingest/reconnect/spool/dedup metric families.
- In-process mock exchange endpoints are available at `/mock/ws/public`, `/mock/ws/private`, `/mock/rest/time`, `/mock/rest/snapshot` when running with `--mock`.
- Scenario controls:
  - `--mock-gap-every N`
  - `--mock-disconnect-every N`
  - `--mock-silence-ms N`
  - `--mock-mongo-down-ms N`
  - `--mock-binary-rate P`
