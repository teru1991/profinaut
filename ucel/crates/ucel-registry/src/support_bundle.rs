use crate::hub::{ExchangeId, Hub};

pub fn hub_bundle(hub: &Hub) -> serde_json::Value {
    let mut venues = Vec::new();
    for exchange in [
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::Coinbase,
        ExchangeId::Coincheck,
        ExchangeId::Deribit,
        ExchangeId::Gmocoin,
        ExchangeId::Kraken,
        ExchangeId::Okx,
        ExchangeId::Upbit,
    ] {
        let ops = hub.list_operations(exchange).unwrap_or_default();
        let channels = hub.list_channels(exchange).unwrap_or_default();
        venues.push(serde_json::json!({
            "exchange_id": exchange.as_str(),
            "operations": ops.len(),
            "channels": channels.len(),
        }));
    }

    serde_json::json!({
        "ssot": {
            "rules_version": "v1",
            "coverage_hash": "unknown"
        },
        "venues": venues
    })
}
