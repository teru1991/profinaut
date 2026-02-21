use futures_util::StreamExt;
use ucel_registry::hub::{ExchangeId, Hub};

#[tokio::main]
async fn main() {
    let hub = Hub::default();
    let mut stream = hub
        .ws(ExchangeId::Binance)
        .subscribe("spot.public.ws.trades", None)
        .await
        .unwrap();

    if let Some(msg) = stream.next().await {
        println!("got message: {}", msg.unwrap().raw.len());
    }
}
