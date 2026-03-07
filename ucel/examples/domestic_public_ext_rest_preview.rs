use ucel_sdk::hub::{ExchangeId, Hub};
use ucel_sdk::DomesticPublicRestExtensionFacade;

fn main() {
    let hub = Hub::default();
    let venues = [
        ExchangeId::Bitbank,
        ExchangeId::Bitflyer,
        ExchangeId::Coincheck,
        ExchangeId::Gmocoin,
        ExchangeId::Bittrade,
        ExchangeId::Sbivc,
    ];

    for venue in venues {
        let facade = DomesticPublicRestExtensionFacade::new(hub.clone(), venue);
        let preview = facade
            .preview_domestic_public_rest_extension_support()
            .expect("preview domestic public rest extension support");
        println!("{}", serde_json::to_string_pretty(&preview).expect("json"));
    }
}
