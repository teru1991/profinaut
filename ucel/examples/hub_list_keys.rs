use ucel_registry::hub::{ExchangeId, Hub};

fn main() {
    let hub = Hub::default();
    let ops = hub.list_operations(ExchangeId::Binance).unwrap();
    let channels = hub.list_channels(ExchangeId::Binance).unwrap();
    println!("ops={} channels={}", ops.len(), channels.len());
}
