# marketdata-rs

Minimal Rust market data service skeleton using Axum.

## Run

```bash
cd services/marketdata-rs
cargo run
```

Server listens on `0.0.0.0:8081`.

## curl examples

```bash
curl -s http://127.0.0.1:8081/healthz | jq
curl -s http://127.0.0.1:8081/capabilities | jq
curl -s "http://127.0.0.1:8081/ticker/latest?exchange=gmo&symbol=BTC_JPY" | jq
```

Invalid requests return HTTP 400 with JSON error payloads:

```bash
curl -i -s "http://127.0.0.1:8081/ticker/latest?exchange=binance&symbol=BTC_JPY"
curl -i -s "http://127.0.0.1:8081/ticker/latest?exchange=gmo&symbol=ETH_JPY"
curl -i -s "http://127.0.0.1:8081/ticker/latest"
```
