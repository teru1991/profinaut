use ucel_registry::hub::Hub;

fn main() {
    let hub = Hub::default();
    for exchange in hub.list_exchanges() {
        let (rest, ws) = hub
            .list_catalog_entries(exchange)
            .expect("catalog entry counts");
        println!("{} rest={} ws={}", exchange.as_str(), rest, ws);
    }
}
