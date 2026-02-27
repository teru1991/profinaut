use ucel_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = SdkConfig::load(None)?;
    let sdk = SdkBuilder::new(cfg).build()?;

    for ex in [
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
        let ops = sdk.list_operations(ex)?;
        println!("{} ops: {}", ex.as_str(), ops.len());
    }
    Ok(())
}
