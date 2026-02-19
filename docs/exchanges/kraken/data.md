# Kraken Data Distribution Catalog (Official)

Fixed columns:
id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url

| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| data.ohlc.minute.json | market-candles | https://api.kraken.com/0/public/OHLC?pair={pair}&interval={minutes}&since={since} | json | none | on-request | rolling API window (per endpoint behavior) | error(req):array<string>,result(req):object(pair=>array<candle>) | official REST endpoint for historical candles; no separate bulk file service in covered source set | https://docs.kraken.com/api/docs/rest-api/get-ohlc-data/ |
