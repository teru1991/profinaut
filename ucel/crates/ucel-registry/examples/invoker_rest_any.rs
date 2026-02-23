use ucel_registry::invoker::{InvocationContext, Invoker, OperationId, VenueId};

#[tokio::main]
async fn main() {
    let invoker = Invoker::default();
    let venue: VenueId = "binance".parse().unwrap();
    let id: OperationId = "spot.public.rest.time".parse().unwrap();
    let _ = invoker
        .rest_call_raw_symbol(&venue, &id, "BTCUSDT", InvocationContext::default())
        .await;
}
