# Serving APIs how-to

## Prerequisites

- `DB_DSN=sqlite:///...` points to marketdata DB.
- Optional: `CLICKHOUSE_DSN=sqlite:///...` for serving sync target.
- Optional: `POSTGRES_OPS_DSN=sqlite:///...` for ops/asset schema bootstrap.

## Materialize Gold + sync serving stores

```bash
curl -sS -X POST 'http://localhost:8000/gold/materialize'
```

Expected response:

```json
{
  "ok": true,
  "ticker_latest_rows": 1,
  "bba_rows": 1,
  "ohlcv_rows": 2,
  "clickhouse_synced_rows": 4
}
```

## Read APIs

### Ticker latest

```bash
curl -sS 'http://localhost:8000/markets/ticker/latest?venue=gmo&symbol=BTC_JPY'
```

```json
{
  "venue": "gmo",
  "symbol": "BTC_JPY",
  "price": 100.5,
  "bid": 100.0,
  "ask": 101.0,
  "as_of": "2026-01-01T00:00:00Z",
  "raw_refs": ["r1"]
}
```

### Best bid/ask latest

```bash
curl -sS 'http://localhost:8000/markets/bba/latest?venue=gmo&symbol=BTC_JPY'
```

### OHLCV 1m range

```bash
curl -sS 'http://localhost:8000/markets/ohlcv?venue=gmo&symbol=BTC_JPY&tf=1m&from=2026-01-01T00:00:00Z&to=2026-01-01T01:00:00Z&limit=100'
```

## Cache and degradation behavior

- Valkey-style cache keys: `ticker_latest:{venue}:{symbol}`, `bba:{venue}:{symbol}`.
- Cache uses TTL + jitter and single-flight stampede protection.
- Gold materialization invalidates affected ticker/bba keys.
- If ClickHouse read fails, API falls back to SQLite gold read.
- If cache is unavailable, read path still works by loading from backend.

## Read metrics endpoint

```bash
curl -sS 'http://localhost:8000/markets/read-metrics'
```

Returns request count, errors, average latency, cache hit/miss totals, and ClickHouse availability flag.
