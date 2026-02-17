# MarketData E2E with fake GMO (DP-035A)

## Purpose

Run deterministic end-to-end verification using a local fake GMO server (REST + WS), then validate:

1. raw data is ingested (`raw_ingest_meta` rows)
2. Gold orderbook BBO endpoint returns expected `found=true` payload

## One-command verification

```bash
scripts/verification/marketdata_e2e_fake_gmo.sh
```

## Expected output

- `bbo_latest={..."found":true...}`
- `raw_ingest_meta_count=<n>` where `n >= 1`
- `E2E fake GMO PASS`
