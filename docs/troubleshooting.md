# Troubleshooting Guide

## Crypto Collector — Persistence Layer

### Mongo Down: What Happens?

When MongoDB is unavailable the `MongoSink` retries each batch up to
`persistence.mongo_max_retries` times with exponential backoff
(`retry_base_ms * 2^attempt` milliseconds).

After exhausting retries the `MongoSink` transitions its state:

| Consecutive batch failures | State |
|---------------------------|-------|
| < `mongo_consecutive_failures_for_degraded` | `MongoUnavailable` |
| ≥ `mongo_consecutive_failures_for_degraded` | `Degraded` |

Any successful write resets the counter and returns to `Ok`.

---

### Durable Spool: Fallback Behaviour

When `persistence.spool.enabled = true`, a batch that fails Mongo is
**written to the local spool directory** instead of being dropped.

```
write_batch(batch)
  → [dedup filter, if enabled]
  → try Mongo
      → Ok:                       written to Mongo, done
      → MongoUnavailable:
          → spool enabled:        written to spool, queued for replay
          → spool disabled:       returns Err (batch lost unless caller retries)
```

#### Spool full: on_full policy

If the spool has reached `persistence.spool.max_total_mb`:

| `on_full` value | Behaviour |
|-----------------|-----------|
| `drop_all` | The entire batch is silently dropped. `spool_dropped_total{exchange,channel}` is incremented. |
| `drop_ticker_depth_keep_trade` | Orderbook / depth / ticker channel envelopes are dropped. Trade channel envelopes block (50 ms sleep + retry) until space is available. |
| `block` | All envelopes block until spool space becomes available. |

---

### Replay Worker: Draining the Spool

The `ReplayWorker` runs as a background Tokio task and automatically
replays spooled data to Mongo once connectivity recovers.

- Replays oldest segments first (ascending file sequence number).
- A segment is **deleted only after** a successful `insert_many` call.
  Power loss mid-replay is safe; the segment is replayed again on restart.
- Rate-limited by `replay.rate_limit_ms` between batches.
- Shutdown-safe: completes the in-flight insert before exiting.

---

### Dedup Window

When `persistence.dedup.enabled = true`, duplicate messages within the
`window_seconds` are dropped before reaching Mongo or the spool.

Dedup key priority (per envelope):
1. `message_id` (if present)
2. `seq:{exchange}:{channel}:{sequence}` (if sequence is present)
3. `hash:{payload_hash}` (DefaultHasher of serialised JSON)

Dropped duplicates increment `dedup_dropped_total{exchange,channel}`.

---

### Manual Verification Steps (Live Mongo)

To verify the real `insert_many` path against a running MongoDB:

1. Start MongoDB: `docker run -d -p 27017:27017 mongo:6`
2. Implement `RealMongoTarget` wrapping `mongodb::Collection<bson::Document>`:
   ```rust
   // In a feature-gated module (e.g. `--features real-mongo`):
   #[async_trait]
   impl MongoTarget for mongodb::Collection<bson::Document> {
       async fn insert_many_envelopes(&self, envelopes: &[Envelope]) -> Result<(), String> {
           let docs: Vec<_> = envelopes.iter()
               .map(|e| bson::to_document(e).unwrap())
               .collect();
           self.insert_many(docs).await.map(|_| ()).map_err(|e| e.to_string())
       }
   }
   ```
3. Build with `cargo build -p crypto-collector --features real-mongo`
4. Configure `collector.toml` with `[persistence]` section
5. Inspect collection: `mongosh --eval 'db.crypto_envelopes.countDocuments()'`

> **Note:** The `mongodb` crate is not compiled by default (marked NOT VERIFIED).
> All functional behaviour is covered by unit tests using `FakeMongoTarget`.

---

### Config Reference (persistence section)

```toml
[persistence]
mongo_uri                          = "mongodb://localhost:27017"
mongo_database                     = "market_data"
mongo_collection                   = "crypto_envelopes"
mongo_max_retries                  = 3
mongo_retry_base_ms                = 100
mongo_consecutive_failures_for_degraded = 3

[persistence.spool]
enabled         = false
dir             = "/var/spool/crypto-collector"
max_segment_mb  = 64
max_total_mb    = 1024
on_full         = "drop_ticker_depth_keep_trade"  # or "drop_all" | "block"

[persistence.dedup]
enabled         = false
window_seconds  = 300
max_keys        = 100000
```

---

## Crypto Collector — Exchange Runtime (Task E)

### Frequent reconnects / URL failover

Symptoms:
- `ws_connected` gauge flaps between 0 and 1.
- Repeating connect/disconnect logs.

Interpretation:
- Runtime is applying exponential backoff with jitter.
- After repeated failures on the current URL, it rotates to the next `ws.connections.urls` entry.

Actions:
1. Validate primary WS endpoint reachability and TLS chain.
2. Confirm fallback URLs are valid and not blocked by firewall.
3. Increase `read_timeout_ms` if exchange heartbeat cadence is lower than expected.

### ACK timeout metric growth

Metric: `subscribe_ack_timeout_total{exchange,connection}`

Meaning:
- Subscribe message was sent but matching ACK did not arrive before `timeout_ms`.
- Runtime retried subscribe with bounded backoff.

Actions:
1. Verify `subscriptions.ack.field`/`value` JSON pointer matcher against actual exchange ACK payload.
2. If multiplexed ACKs are used, set `correlation_pointer` to a stable request identifier.
3. Increase `timeout_ms` if exchange ACK latency is intermittently high.

### Time quality metrics interpretation

Collected values:
- `server_time presence ratio`: share of received messages carrying extractable server time.
- `clock_skew_ms`: local clock minus server time.
- `end_to_end_lag_ms`: non-negative transport/processing lag estimate.

If skew drifts significantly:
- verify host NTP synchronization,
- compare across multiple exchanges to isolate local vs remote clock issues.

## Task F Mock runtime troubleshooting

- If `/metrics` is static in mock mode, verify `--mock` is enabled and confirm `MOCK_ENABLED=1` in process env.
- If reconnect metrics do not move, lower `--mock-disconnect-every` (e.g. 4) to force frequent reconnect transitions.
- If spool behavior is not visible, raise `--mock-mongo-down-ms` so spool grows long enough before replay drain.
