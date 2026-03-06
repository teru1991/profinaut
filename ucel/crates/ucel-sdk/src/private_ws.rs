use ucel_core::PrivateWsChannel;
use ucel_registry::hub::{ExchangeId, Hub, HubError, WsHub};

pub trait PrivateWsClient {
    fn subscribe_private_balances(&self) -> Result<(), HubError>;
    fn subscribe_private_orders(&self) -> Result<(), HubError>;
    fn subscribe_private_fills(&self) -> Result<(), HubError>;
    fn subscribe_private_positions(&self) -> Result<(), HubError>;
    fn subscribe_private_session(&self) -> Result<(), HubError>;
}

pub struct PrivateWsFacade {
    ws: WsHub,
}

impl PrivateWsFacade {
    pub fn new(hub: &Hub, exchange: ExchangeId) -> Self {
        Self { ws: hub.ws(exchange) }
    }

    pub fn preview_private_ws_plan(exchange: ExchangeId, channels: &[PrivateWsChannel]) -> String {
        let joined = channels
            .iter()
            .map(|c| format!("{c:?}").to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(",");
        format!("exchange={} channels={}", exchange.as_str(), joined)
    }

    fn subscribe_channel(&self, channel: &str) -> Result<(), HubError> {
        let _ = &self.ws;
        if channel.is_empty() {
            return Err(HubError::RegistryValidation("empty private channel".into()));
        }
        Ok(())
    }
}

impl PrivateWsClient for PrivateWsFacade {
    fn subscribe_private_balances(&self) -> Result<(), HubError> {
        self.subscribe_channel("private_balances")
    }

    fn subscribe_private_orders(&self) -> Result<(), HubError> {
        self.subscribe_channel("private_orders")
    }

    fn subscribe_private_fills(&self) -> Result<(), HubError> {
        self.subscribe_channel("private_fills")
    }

    fn subscribe_private_positions(&self) -> Result<(), HubError> {
        self.subscribe_channel("private_positions")
    }

    fn subscribe_private_session(&self) -> Result<(), HubError> {
        self.subscribe_channel("private_session")
    }
}


pub fn ingest_runtime_hint() -> &'static str {
    "private ingest requires reauth-then-resubscribe on restart"
}
