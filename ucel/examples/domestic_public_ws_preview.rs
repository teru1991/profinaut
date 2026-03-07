use ucel_registry::hub::{ws::list_pending_vendor_public_ws_extension_channel_ids, ExchangeId};

fn main() {
    let venues = [
        ExchangeId::Bitbank,
        ExchangeId::Bitflyer,
        ExchangeId::Coincheck,
        ExchangeId::Gmocoin,
        ExchangeId::Bittrade,
        ExchangeId::Sbivc,
    ];

    for venue in venues {
        let pending = list_pending_vendor_public_ws_extension_channel_ids(venue)
            .expect("list pending vendor extension channels");
        println!(
            "venue={} pending_vendor_extension_009e={} channels={:?}",
            venue.as_str(),
            pending.len(),
            pending
        );
    }
}
