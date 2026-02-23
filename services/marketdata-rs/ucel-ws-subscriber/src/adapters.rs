use std::sync::Arc;
use ucel_transport::ws::adapter::WsVenueAdapter;

use ucel_cex_gmocoin::ws::GmoCoinWsAdapter;

pub fn create(exchange_id: &str) -> Option<Arc<dyn WsVenueAdapter>> {
    match exchange_id {
        "gmocoin" => Some(Arc::new(GmoCoinWsAdapter::new())),
        _ => None,
    }
}
