use std::sync::Arc;
use ucel_transport::ws::adapter::WsVenueAdapter;

// ---------- GMO (split public/private) ----------
use ucel_cex_gmocoin::rest::GmoCredentials;
use ucel_cex_gmocoin::ws::{GmoCoinPrivateWsAdapter, GmoCoinPublicWsAdapter};

// ---------- Existing exchanges ----------
use ucel_cex_bybit::ws::BybitWsAdapter;
use ucel_cex_bitget::ws::BitgetSpotWsAdapter;
use ucel_cex_okx::ws::OkxWsAdapter;
use ucel_cex_kraken::ws::KrakenSpotWsAdapter;
use ucel_cex_htx::ws::HtxSpotWsAdapter;
use ucel_cex_bittrade::ws::BitTradeWsAdapter;

// Binance split adapters
use ucel_cex_binance::ws::BinanceSpotWsAdapter;
use ucel_cex_binance_usdm::ws::BinanceUsdmWsAdapter;
use ucel_cex_binance_coinm::ws::BinanceCoinmWsAdapter;
use ucel_cex_binance_options::ws::BinanceOptionsWsAdapter;

// ---------- Future JP exchanges (placeholders) ----------
// When you add these crates later, enable via Cargo features and uncomment the uses.
// use ucel_cex_coincheck::ws::{CoincheckPublicWsAdapter, CoincheckPrivateWsAdapter};
// use ucel_cex_bitflyer::ws::{BitflyerPublicWsAdapter, BitflyerPrivateWsAdapter};
// use ucel_cex_bitbank::ws::{BitbankPublicWsAdapter, BitbankPrivateWsAdapter};
// use ucel_cex_sbivc::ws::{SbivcPublicWsAdapter, SbivcPrivateWsAdapter};

pub fn create(exchange_id: &str) -> Result<Arc<dyn WsVenueAdapter>, String> {
    match exchange_id {
        // ----------------------------
        // GMO Coin
        // ----------------------------
        "gmocoin-public" => Ok(Arc::new(GmoCoinPublicWsAdapter::new())),

        "gmocoin-private" => {
            let api_key = std::env::var("GMO_API_KEY")
                .map_err(|_| "missing env GMO_API_KEY (required for gmocoin-private)".to_string())?;
            let api_secret = std::env::var("GMO_API_SECRET")
                .map_err(|_| "missing env GMO_API_SECRET (required for gmocoin-private)".to_string())?;
            let creds = GmoCredentials { api_key, api_secret };
            let a = GmoCoinPrivateWsAdapter::new(creds)?;
            Ok(Arc::new(a))
        }

        // Backward compatibility: "gmocoin" => gmocoin-public
        "gmocoin" => Ok(Arc::new(GmoCoinPublicWsAdapter::new())),

        // ----------------------------
        // Bybit
        // ----------------------------
        "bybit-spot" => Ok(Arc::new(BybitWsAdapter::spot())),
        "bybit-linear" => Ok(Arc::new(BybitWsAdapter::linear())),
        "bybit-inverse" => Ok(Arc::new(BybitWsAdapter::inverse())),
        "bybit-options" => Ok(Arc::new(BybitWsAdapter::option())),

        // ----------------------------
        // Bitget
        // ----------------------------
        "bitget-spot" => Ok(Arc::new(BitgetSpotWsAdapter::new())),

        // ----------------------------
        // OKX
        // ----------------------------
        "okx-spot" => Ok(Arc::new(OkxWsAdapter::spot())),
        "okx-swap" => Ok(Arc::new(OkxWsAdapter::swap())),
        "okx-futures" => Ok(Arc::new(OkxWsAdapter::futures())),
        "okx-option" => Ok(Arc::new(OkxWsAdapter::option())),

        // ----------------------------
        // Others
        // ----------------------------
        "kraken" => Ok(Arc::new(KrakenSpotWsAdapter::new())),
        "htx-spot" => Ok(Arc::new(HtxSpotWsAdapter::new())),
        "bittrade" => Ok(Arc::new(BitTradeWsAdapter::new())),

        // ----------------------------
        // Binance split
        // ----------------------------
        "binance-spot" => Ok(Arc::new(BinanceSpotWsAdapter::new())),
        "binance-usdm" => Ok(Arc::new(BinanceUsdmWsAdapter::new())),
        "binance-coinm" => Ok(Arc::new(BinanceCoinmWsAdapter::new())),
        "binance-options" => Ok(Arc::new(BinanceOptionsWsAdapter::new())),

        // ----------------------------
        // Future JP exchanges (pre-wired IDs)
        // These are intentionally "known IDs" so coverage/rules can be prepared now.
        // When you add the crates, switch these to real adapters.
        // ----------------------------
        "coincheck-public" | "coincheck-private" |
        "bitflyer-public" | "bitflyer-private" |
        "bitbank-public" | "bitbank-private" |
        "sbivc-public" | "sbivc-private" => {
            Err(format!("adapter not implemented yet for {exchange_id} (JP venue placeholder is pre-registered)"))
        }

        _ => Err(format!("no adapter registered for exchange_id={exchange_id}")),
    }
}