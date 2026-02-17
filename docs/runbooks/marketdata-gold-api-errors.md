# MarketData Gold API: Common errors and fixes (DP-034A)

## Error response shape

Gold API validation/read errors use a common envelope:

```json
{
  "code": "<ERROR_CODE>",
  "message": "<human readable>",
  "details": {"...": "..."},
  "request_id": "..."
}
```

## Common input errors

- `MISSING_REQUIRED_QUERY`
  - Cause: required query parameters are missing.
  - Fix: provide all required params, e.g. `venue_id` + `market_id`, or ticker triplet `venue_id` + `market_id` + `instrument_id`.

- `INVALID_TIMEFRAME`
  - Cause: unsupported `tf` value.
  - Fix: use one of `1m`, `5m`, `15m`, `1h`, `1d` (aliases like `1min` are accepted where supported).

- `INVALID_TIMESTAMP`
  - Cause: `from`/`to` is not RFC3339.
  - Fix: send RFC3339 timestamps (example: `2026-02-16T00:00:00Z`).

- `INVALID_TIME_RANGE`
  - Cause: `to < from`.
  - Fix: ensure `to` is greater than or equal to `from`.

## Read model errors

- `READ_MODEL_UNAVAILABLE`
  - Cause: DB is unavailable or not configured.
  - Fix: verify `DB_DSN`, DB process health, file permissions, and connectivity.
