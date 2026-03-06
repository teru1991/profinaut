use ucel_registry::hub::Hub;

fn main() {
    let hub = Hub::default();
    let exchanges = hub.list_exchanges();
    for exchange in exchanges {
        let cap = hub.capabilities(exchange).expect("capabilities");
        let scope = cap
            .venue_access
            .as_ref()
            .map(|v| format!("{:?}", v.scope))
            .unwrap_or_else(|| "None".into());
        println!("{} venue_access={}", exchange.as_str(), scope);
    }
}
