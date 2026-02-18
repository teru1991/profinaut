use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Stable ingestion envelope v1 for crypto collector subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub envelope_version: u16,
    pub adapter_version: String,
    pub connector_instance_id: String,

    pub exchange: String,
    pub symbol: String,
    pub channel: String,
    pub channel_detail: Option<String>,

    pub server_time: Option<i64>,
    pub local_time_ns: u64,
    pub sequence: Option<u64>,
    pub message_id: Option<String>,

    pub payload: Value,
}

/// Returns local monotonic wall-clock timestamp in nanoseconds since UNIX epoch.
pub fn now_local_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct EnvelopeBuilder {
    adapter_version: String,
    connector_instance_id: String,
    exchange: String,
    symbol: String,
    channel: String,
    payload: Value,
    channel_detail: Option<String>,
    server_time: Option<i64>,
    local_time_ns: Option<u64>,
    sequence: Option<u64>,
    message_id: Option<String>,
}

impl Envelope {
    pub fn builder(
        adapter_version: impl Into<String>,
        connector_instance_id: impl Into<String>,
        exchange: impl Into<String>,
        symbol: impl Into<String>,
        channel: impl Into<String>,
        payload: Value,
    ) -> EnvelopeBuilder {
        EnvelopeBuilder {
            adapter_version: adapter_version.into(),
            connector_instance_id: connector_instance_id.into(),
            exchange: exchange.into(),
            symbol: symbol.into(),
            channel: channel.into(),
            payload,
            channel_detail: None,
            server_time: None,
            local_time_ns: None,
            sequence: None,
            message_id: None,
        }
    }
}

impl EnvelopeBuilder {
    pub fn channel_detail(mut self, channel_detail: impl Into<String>) -> Self {
        self.channel_detail = Some(channel_detail.into());
        self
    }

    pub fn server_time(mut self, server_time: i64) -> Self {
        self.server_time = Some(server_time);
        self
    }

    pub fn local_time_ns(mut self, local_time_ns: u64) -> Self {
        self.local_time_ns = Some(local_time_ns);
        self
    }

    pub fn sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn message_id(mut self, message_id: impl Into<String>) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    pub fn build(self) -> Envelope {
        Envelope {
            envelope_version: 1,
            adapter_version: self.adapter_version,
            connector_instance_id: self.connector_instance_id,
            exchange: self.exchange,
            symbol: self.symbol,
            channel: self.channel,
            channel_detail: self.channel_detail,
            server_time: self.server_time,
            local_time_ns: self.local_time_ns.unwrap_or_else(now_local_time_ns),
            sequence: self.sequence,
            message_id: self.message_id,
            payload: self.payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builder_sets_required_fields_and_version() {
        let env = Envelope::builder(
            "binance@1.0.0",
            "connector-uuid",
            "binance-main",
            "BTCUSDT",
            "trade",
            json!({"p":"1"}),
        )
        .build();

        assert_eq!(env.envelope_version, 1);
        assert_eq!(env.adapter_version, "binance@1.0.0");
        assert_eq!(env.channel, "trade");
        assert!(env.local_time_ns > 0);
    }
}
