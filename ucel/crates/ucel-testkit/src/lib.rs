use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use ucel_core::ErrorCode;
use ucel_transport::{HealthLevel, HealthStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    RestRespond {
        path: String,
        status: u16,
        body: String,
        headers: Vec<(String, String)>,
    },
    RestRateLimit429 {
        path: String,
        retry_after_ms: u64,
    },
    WsAccept,
    WsSendJson(String),
    WsDropConnection,
    SleepMs(u64),
    InjectOrderBookGap,
    InjectOutOfOrder,
    InjectDuplicate,
    ExpectErrorCode(ErrorCode),
    ExpectMetricInc(String),
    ExpectDegraded,
}

#[derive(Debug, Default)]
pub struct RestMockServer {
    pub responses: VecDeque<(u16, String)>,
}

impl RestMockServer {
    pub fn enqueue(&mut self, status: u16, body: impl Into<String>) {
        self.responses.push_back((status, body.into()));
    }

    pub fn next_response(&mut self) -> Option<(u16, String)> {
        self.responses.pop_front()
    }
}

#[derive(Debug, Default)]
pub struct WsMockServer {
    pub accepted: bool,
    pub events: VecDeque<String>,
    pub dropped: bool,
}

impl WsMockServer {
    pub fn accept(&mut self) {
        self.accepted = true;
    }

    pub fn send_json(&mut self, msg: impl Into<String>) {
        self.events.push_back(msg.into());
    }

    pub fn drop_connection(&mut self) {
        self.dropped = true;
    }
}

pub fn degraded_health(reason: &str, code: ErrorCode) -> HealthStatus {
    HealthStatus {
        level: HealthLevel::Degraded,
        degraded_reason: Some(reason.into()),
        last_success_ts: None,
        last_error_code: Some(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucel_core::ResolvedSecret;

    #[test]
    fn mock_rest_supports_response_queue() {
        let mut server = RestMockServer::default();
        server.enqueue(200, "ok");
        server.enqueue(429, "rate limited");
        assert_eq!(server.next_response(), Some((200, "ok".into())));
        assert_eq!(server.next_response(), Some((429, "rate limited".into())));
    }

    #[test]
    fn ws_mock_supports_drops() {
        let mut ws = WsMockServer::default();
        ws.accept();
        ws.send_json("{\"type\":\"trade\"}");
        ws.drop_connection();
        assert!(ws.accepted);
        assert!(ws.dropped);
    }

    #[test]
    fn resolved_secret_masking_is_enforced() {
        let s = ResolvedSecret {
            api_key: "my-key".into(),
            api_secret: Some("my-secret".into()),
            passphrase: Some("my-pass".into()),
        };
        let dbg = format!("{s:?}");
        let disp = format!("{s}");
        assert!(!dbg.contains("my-secret"));
        assert!(!disp.contains("my-pass"));
        assert!(disp.contains("***"));
    }
}
