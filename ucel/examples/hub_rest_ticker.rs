use ucel_registry::hub::{ExchangeId, Hub};

#[tokio::main]
async fn main() {
    let hub = Hub::default();
    let _ = hub
        .rest(ExchangeId::Binance)
        .call("spot.public.rest.ticker.24hr", Some(&[("symbol", "BTCUSDT")]), None)
        .await
        .unwrap();
}
