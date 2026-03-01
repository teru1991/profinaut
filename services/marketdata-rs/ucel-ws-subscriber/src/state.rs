use std::sync::Arc;

use ucel_transport::health::TransportHealth;
use ucel_transport::obs::{StabilityEventRing, TransportMetrics};

#[derive(Clone)]
pub struct AppState {
    pub exchange_id: String,
    pub conn_id: String,
    pub metrics: Arc<TransportMetrics>,
    pub events: Arc<StabilityEventRing>,
    pub health: Arc<parking_lot::RwLock<TransportHealth>>,
    pub rules_snapshot: Arc<parking_lot::RwLock<serde_json::Value>>,
}

impl AppState {
    pub fn new(exchange_id: String, conn_id: String) -> Self {
        Self {
            exchange_id,
            conn_id,
            metrics: TransportMetrics::new(),
            events: StabilityEventRing::new(512),
            health: Arc::new(parking_lot::RwLock::new(TransportHealth::healthy())),
            rules_snapshot: Arc::new(parking_lot::RwLock::new(serde_json::json!({}))),
        }
    }
}
