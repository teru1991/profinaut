use crate::obs::logging::ObsRequiredKeys;
use crate::obs::{StabilityEventRing, TransportMetrics};
use async_trait::async_trait;
use serde_json::Value;
use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone)]
pub struct OutboundMsg {
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum InboundClass {
    Data {
        op_id: Option<String>,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
    },
    Ack {
        op_id: String,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
    },
    Nack {
        reason: String,
        op_id: Option<String>,
        symbol: Option<String>,
        params_canon_hint: Option<String>,
        /// Optional: server hints when to retry (ms).
        /// If provided, transport will apply limiter penalty automatically.
        retry_after_ms: Option<u64>,
    },
    /// HTX/BitTrade app ping -> immediate pong send
    Respond {
        msg: OutboundMsg,
    },
    System,
    Unknown,
}

#[async_trait]
pub trait WsVenueAdapter: Send + Sync + 'static {
    fn exchange_id(&self) -> &str;
    fn ws_url(&self) -> String;
    async fn fetch_symbols(&self) -> Result<Vec<String>, String>;
    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        params: &Value,
    ) -> Result<Vec<OutboundMsg>, String>;
    fn classify_inbound(&self, raw: &[u8]) -> InboundClass;

    /// periodic app ping (Bybit/Bitget/Kraken/OKX)
    fn ping_msg(&self) -> Option<OutboundMsg> {
        None
    }
}

use crate::security::{check_json_limits, JsonLimits};

#[derive(Debug, Clone, Copy, Default)]
pub struct InboundJsonGuard {
    pub limits: JsonLimits,
}

impl InboundJsonGuard {
    pub fn enforce(&self, bytes: &[u8]) -> Result<(), UcelError> {
        check_json_limits(bytes, self.limits)
            .map_err(|e| UcelError::new(ErrorCode::WsProtocolViolation, e.message))
    }
}

pub fn inbound_violation(msg: impl Into<String>) -> UcelError {
    UcelError::new(ErrorCode::WsProtocolViolation, msg.into())
}

pub fn record_decode_error(
    metrics: &TransportMetrics,
    ring: &StabilityEventRing,
    required: &ObsRequiredKeys,
    venue: &'static str,
    op: &str,
    symbol: &str,
) {
    metrics.on_decode_error();
    ring.push_required(
        "decode_error",
        serde_json::json!({"venue": venue, "op": op}),
        &required.exchange_id,
        &required.conn_id,
        &required.run_id,
        op,
        symbol,
    );
}
