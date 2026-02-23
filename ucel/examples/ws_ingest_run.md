# ws-ingest run (GMO example)

```bash
# repository root
export UCEL_COVERAGE_DIR="ucel/coverage"
export UCEL_RULES_DIR="ucel/crates/ucel-ws-rules/rules"
export UCEL_STORE_PATH="/tmp/ucel-ws-subscriber.sqlite"
export UCEL_JOURNAL_DIR="/tmp/ucel-wal"
export UCEL_FSYNC_MODE="relaxed"        # dev: relaxed, production: balanced recommended
export UCEL_RECV_QUEUE_CAP="4096"
export UCEL_MAX_FRAME_BYTES="4194304"
export UCEL_MAX_INFLIGHT_PER_CONN="64"
export UCEL_CONNECT_TIMEOUT_SECS="10"
export UCEL_IDLE_TIMEOUT_SECS="30"
export UCEL_RECONNECT_STORM_WINDOW_SECS="30"
export UCEL_RECONNECT_STORM_MAX="12"
export UCEL_MAX_CONNECTIONS_PER_EXCHANGE="512"
export UCEL_ENABLE_PRIVATE_WS="false"
export RUST_LOG="info"

cargo run -p ucel-ws-subscriber
```

Single exchange allowlist is currently configured in code via `IngestConfig.exchange_allowlist`.

## Checklist

### 1) Startup (within 5 minutes)
- Log contains `ucel-ws-subscriber starting`.
- Log contains symbols + ops counts.
- `UCEL_STORE_PATH` sqlite file is created.
- `subscriptions` table exists.
- `state='pending'` entries start progressing to `inflight`/`active`.
- WAL files are created under `UCEL_JOURNAL_DIR` and keep growing.
- Example check: `ls -lh /tmp/ucel-wal` shows updated timestamps/sizes.

### 2) During run (around 15 minutes)
- `active` does not stay at `0` (at least ticker becomes active).
- Deadletter does not increase rapidly.
- Watch for op_id mismatch / symbol mapping / subscribe errors.
- CPU is stable (receiver loop should not do heavy work).

### 3) Reconnect behavior (manual)
- Temporarily cut network (or emulate endpoint outage) and confirm connect error + reconnecting logs.
- After recovery, reconnect succeeds and WAL growth resumes.
- Store state moves `active/inflight -> pending -> active`.

### 4) Safety stop behavior (intentional)
- Start with very small `UCEL_MAX_FRAME_BYTES`, confirm frame-too-large stop behavior.
- Make WAL directory unwritable/full and confirm process stops on append-first failure.
