# Silver recompute / diff how-to

## Purpose

This runbook describes deterministic Silver recompute from Bronze JSONL.gz partitions and deterministic diffing between two Silver outputs.

## Bronze layout expected

Bronze scanner reads:

- `bronze/crypto/<venue>/<YYYY>/<MM>/<DD>/<HH>/*.jsonl.gz`

Time range is resolved by hour (`dt/hh`) and can be filtered by venue, symbol, and event type.

## Commands

### 1) Recompute Silver (backfill)

```bash
python -m services.marketdata.app.cli silver backfill \
  --bronze-root /data/lake \
  --silver-root /data/silver-run-a \
  --from-ts 2026-02-16T00:00:00Z \
  --to-ts 2026-02-16T23:00:00Z \
  --venue gmo \
  --symbol BTC_JPY \
  --event-type trade \
  --event-type ticker
```

Expected output: JSON report with `row_counts`, `rejection_counts`, `sample_hashes`, and `latency_ms`.

### 2) Recompute with small-file mitigation (compaction mode)

```bash
python -m services.marketdata.app.cli silver backfill \
  --bronze-root /data/lake \
  --silver-root /data/silver-run-b \
  --from-ts 2026-02-16T00:00:00Z \
  --to-ts 2026-02-16T23:00:00Z \
  --venue gmo \
  --compact
```

`--compact` raises effective part size to reduce small files.

### 3) Diff two runs

```bash
python -m services.marketdata.app.cli silver diff \
  --baseline-silver-root /data/silver-run-a \
  --candidate-silver-root /data/silver-run-b
```

Expected output: deterministic hashes per table and `mismatches` map.

- Empty `mismatches` means outputs are equivalent.
- Non-empty `mismatches` includes per-table baseline/candidate hash values.

## Deterministic diff strategy

- Read all Parquet parts for each table.
- Sort rows by `ts_recv`, then `raw_ref`.
- Canonicalize each row as sorted-key JSON.
- SHA-256 hash canonical row stream per table.

This makes recompute and diff stable across reruns when Bronze input is unchanged.

## Rejections and no-drop behavior

Parse and normalization failures are emitted to `silver/rejections` with:

- `raw_ref`
- `reason_code`
- safe metadata (`venue`, `symbol`, `dt`)

Rejections are counted in recompute output under `row_counts.rejections` and `rejection_counts`.
