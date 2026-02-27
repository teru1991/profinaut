use ucel_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = SdkConfig::load(None)?;
    let sdk = SdkBuilder::new(cfg).build()?;

    let ex = ExchangeId::Binance;
    let ch = sdk.list_channels(ex)?;
    println!("{} channels: {}", ex.as_str(), ch.len());
    Ok(())
}
