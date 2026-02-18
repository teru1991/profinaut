# Market Data Collector Framework v1.4 (Crypto Subsystem)

## Task C — Ingestion Pipeline SSOT Notes

### Envelope v1 invariants (stable)

`Envelope` is fixed to the following schema for the crypto collector subsystem and must not be changed in later tasks:

- `envelope_version: u16` (fixed to `1`)
- `adapter_version: String` (`descriptor.name@descriptor.version`)
- `connector_instance_id: String` (UUID stable for process lifetime)
- `exchange: String` (collector.toml exchange instance name)
- `symbol: String` (canonical symbol)
- `channel: String` (canonical channel)
- `channel_detail: Option<String>`
- `server_time: Option<i64>` (exchange event time; exchange units documented by adapter)
- `local_time_ns: u64` (receipt timestamp in Unix epoch nanoseconds)
- `sequence: Option<u64>`
- `message_id: Option<String>`
- `payload: serde_json::Value` (raw JSON)

Helper contracts:
- `now_local_time_ns() -> u64` returns Unix epoch nanoseconds using `SystemTime::now()`.
- Builder requires all required fields and auto-fills `envelope_version=1` and `local_time_ns` if not provided.

### Batching behavior

The Task C ingestion buffer uses a bounded Tokio MPSC queue and a single consumer runner:

- Producers enqueue `Envelope` into bounded MPSC.
- Consumer batches into `Vec<Envelope>` and emits to `Sink` when either:
  - `max_batch_items` reached, or
  - `max_batch_interval_ms` elapsed.
- `flush()` forces immediate drain of current in-memory batch.
- `shutdown()` stops pipeline control loop and drains remaining queue + batch before exit.

### Backpressure policies

Channel policies are deterministic and channel-specific:

- `trade`: `no_drop` default. On full channel, fail fast (`TradeOverflow`) and increment:
  - `trade_overflow_total{exchange}`
  - `ingest_errors_total{exchange}`
- `ticker`: `drop_old_keep_latest` policy class (Task C skeleton currently applies deterministic drop-on-full at ingress and records drops).
- `depth`: `drop_old_deltas_best_effort` policy class (Task C skeleton currently applies deterministic drop-on-full at ingress and records drops).

Metrics defined in Task C skeleton:

- `ingest_messages_total{exchange,channel}`
- `ingest_errors_total{exchange}`
- `buffer_depth{exchange}`
- `drop_count{exchange,channel}`
- `trade_overflow_total{exchange}`

Implications:
- Trade streams prefer safety and explicit failure over silent loss.
- Ticker/depth streams prioritize pipeline continuity under pressure with explicit observability through `drop_count`.
