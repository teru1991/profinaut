use std::sync::Arc;
use ucel_transport::ws::adapter::WsVenueAdapter;

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;

use ucel_cex_bybit::ws::BybitWsAdapter;
use ucel_cex_bitget::ws::BitgetSpotWsAdapter;
use ucel_cex_okx::ws::OkxWsAdapter;
use ucel_cex_kraken::ws::KrakenSpotWsAdapter;
use ucel_cex_htx::ws::HtxSpotWsAdapter;
use ucel_cex_bittrade::ws::BitTradeWsAdapter;

pub fn create(exchange_id: &str) -> Option<Arc<dyn WsVenueAdapter>> {
    match exchange_id {
        "gmocoin" => Some(Arc::new(GmoCoinWsAdapter::new())),

        "bybit-spot" => Some(Arc::new(BybitWsAdapter::spot())),
        "bybit-linear" => Some(Arc::new(BybitWsAdapter::linear())),
        "bybit-inverse" => Some(Arc::new(BybitWsAdapter::inverse())),
        "bybit-options" => Some(Arc::new(BybitWsAdapter::option())),

        "bitget-spot" => Some(Arc::new(BitgetSpotWsAdapter::new())),

        "okx-spot" => Some(Arc::new(OkxWsAdapter::spot())),
        "okx-swap" => Some(Arc::new(OkxWsAdapter::swap())),
        "okx-futures" => Some(Arc::new(OkxWsAdapter::futures())),
        "okx-option" => Some(Arc::new(OkxWsAdapter::option())),

        "kraken" => Some(Arc::new(KrakenSpotWsAdapter::new())),

        "htx-spot" => Some(Arc::new(HtxSpotWsAdapter::new())),

        "bittrade" => Some(Arc::new(BitTradeWsAdapter::new())),

        _ => None,
    }
}
