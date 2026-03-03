use std::{collections::HashMap, sync::Arc};
use tokio::sync::{watch, Mutex};
use ucel_symbol_adapter::ResyncHint;

#[derive(Clone, Debug, Default)]
pub struct ResyncCoordinatorState {
    pub last_error: Option<&'static str>,
    pub last_hint: Option<&'static str>,
}

type ExchangeId = String;

pub struct ResyncCoordinator {
    inner: Mutex<Inner>,
}

struct Inner {
    receivers: HashMap<ExchangeId, watch::Receiver<Option<ResyncHint>>>,
    state: ResyncCoordinatorState,
}

impl Default for ResyncCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ResyncCoordinator {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                receivers: HashMap::new(),
                state: ResyncCoordinatorState::default(),
            }),
        }
    }

    pub async fn register_resync_receiver(
        &self,
        exchange_id: impl Into<String>,
        rx: watch::Receiver<Option<ResyncHint>>,
    ) {
        let mut g = self.inner.lock().await;
        g.receivers.insert(exchange_id.into(), rx);
    }

    pub async fn snapshot(&self) -> ResyncCoordinatorState {
        self.inner.lock().await.state.clone()
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            {
                let mut g = self.inner.lock().await;
                let mut computed_hint: Option<&'static str> = None;
                for rx in g.receivers.values_mut() {
                    if rx.has_changed().unwrap_or(false) {
                        let hint = rx.borrow_and_update().clone();
                        computed_hint = match hint {
                            Some(ResyncHint::Lagged { .. }) => Some("lagged"),
                            Some(ResyncHint::Reset { .. }) => Some("reset"),
                            None => computed_hint,
                        };
                    }
                }
                if computed_hint.is_some() {
                    g.state.last_hint = computed_hint;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    pub async fn set_error(&self, e: &'static str) {
        self.inner.lock().await.state.last_error = Some(e);
    }

    pub async fn clear_error(&self) {
        self.inner.lock().await.state.last_error = None;
    }
}
