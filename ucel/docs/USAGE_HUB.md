# UCEL Hub Usage

```rust
use ucel_registry::hub::{ExchangeId, Hub};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hub = Hub::default();

    let operations = hub.list_operations(ExchangeId::Binance)?;
    println!("binance ops: {}", operations.len());

    let response = hub
        .rest(ExchangeId::Binance)
        .call("spot.public.rest.ticker.24hr", Some(&[("symbol", "BTCUSDT")]), None)
        .await?;
    println!("status={}", response.status);

    Ok(())
}
```

WS subscriptions use the same key lookup pattern:

```rust
let mut stream = hub
    .ws(ExchangeId::Binance)
    .subscribe("spot.public.ws.trades", None)
    .await?;
```
