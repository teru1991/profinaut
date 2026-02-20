# Data Platform Samples

## BronzeRecord JSON sample
```json
{
  "event_type": "trade",
  "source": {
    "exchange": "gmo",
    "venue": "gmo",
    "channel": "ws",
    "transport": "ws",
    "asset_class": "crypto",
    "op_name": "trade",
    "catalog_id": "btc_jpy"
  },
  "source_event_id": "tr-42",
  "canonical_id": "tr-42",
  "idempotency_key": "gmo:spot:tr-42",
  "event_time": "2026-01-01T00:00:01Z",
  "ingested_at": "2026-01-01T00:00:01Z",
  "payload": {"symbol": "BTC_JPY", "price": 101.2, "qty": 0.1, "side": "buy", "trade_id": 42},
  "meta": {"schema_version": "bronze.v1", "raw_ref": "raw://bronze/gmo/trade/dt=2026-01-01/hh=00/part-*.jsonl.gz#gmo:spot:tr-42"}
}
```

## RawRef examples
- Trade: `raw://bronze/gmo/trade/dt=2026-01-01/hh=00/part-0001.jsonl.gz#gmo:spot:tr-42`
- Ticker: `raw://bronze/gmo/ticker/dt=2026-01-01/hh=00/part-0001.jsonl.gz#gmo:spot:tk-42`

## SQL samples
```sql
SELECT COUNT(*) FROM md_trades;
SELECT COUNT(*) FROM md_best_bid_ask;
SELECT COUNT(*) FROM md_ohlcv;
SELECT COUNT(*) FROM gold_ticker_latest;
SELECT venue_id, instrument_id, price, ts_recv FROM gold_ticker_latest ORDER BY ts_recv DESC LIMIT 10;
```

## Valkey/Cache key shapes
- `ticker_latest:{venue}:{symbol}`
- `bba:{venue}:{symbol}`

Example:
- `ticker_latest:gmo:BTC_JPY`
- `bba:gmo:BTC_JPY`
