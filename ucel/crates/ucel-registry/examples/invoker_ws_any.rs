use futures_util::StreamExt;
use ucel_registry::invoker::{InvocationContext, Invoker, OperationId, VenueId};

#[tokio::main]
async fn main() {
    let invoker = Invoker::default();
    let venue: VenueId = "binance".parse().unwrap();
    let id: OperationId = "spot.public.ws.trades".parse().unwrap();
    if let Ok(mut s) = invoker
        .ws_subscribe_raw_symbol(&venue, &id, "btcusdt", InvocationContext::default())
        .await
    {
        let _ = s.next().await;
    }
}
