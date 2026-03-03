use crate::config::AppConfig;
use crate::resync_loop::{ResyncCoordinator, ResyncCoordinatorState};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    Degraded { reason: &'static str },
}

#[derive(Clone, Debug)]
pub struct HealthSnapshot {
    pub status: HealthStatus,
}

#[derive(Clone)]
pub struct AppState {
    pub cfg: AppConfig,
    pub health_tx: watch::Sender<HealthSnapshot>,
    pub health_rx: watch::Receiver<HealthSnapshot>,
    pub resync: Arc<ResyncCoordinator>,
}

impl AppState {
    pub fn new(cfg: AppConfig) -> Self {
        let (health_tx, health_rx) = watch::channel(HealthSnapshot {
            status: HealthStatus::Degraded { reason: "starting" },
        });
        Self {
            cfg,
            health_tx,
            health_rx,
            resync: Arc::new(ResyncCoordinator::new()),
        }
    }

    pub fn set_ok(&self) {
        let _ = self.health_tx.send(HealthSnapshot {
            status: HealthStatus::Ok,
        });
    }

    pub fn set_degraded(&self, reason: &'static str) {
        let _ = self.health_tx.send(HealthSnapshot {
            status: HealthStatus::Degraded { reason },
        });
    }
}

pub struct Supervisor {
    handles: Mutex<Vec<JoinHandle<()>>>,
    stop_tx: watch::Sender<bool>,
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Supervisor {
    pub fn new() -> Self {
        let (stop_tx, _stop_rx) = watch::channel(false);
        Self {
            handles: Mutex::new(vec![]),
            stop_tx,
        }
    }

    pub async fn spawn_workers(self: Arc<Self>, st: AppState) {
        if st.cfg.exchanges.is_empty() {
            st.set_degraded("no_exchanges");
            return;
        }
        st.set_ok();

        let mut stop_rx = self.stop_tx.subscribe();
        let resync = st.resync.clone();
        let health_tx = st.health_tx.clone();

        let h = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                        let s: ResyncCoordinatorState = resync.snapshot().await;
                        if s.last_error.is_some() {
                            let _ = health_tx.send(HealthSnapshot { status: HealthStatus::Degraded { reason: "snapshot_failed" } });
                        } else if s.last_hint == Some("lagged") {
                            let _ = health_tx.send(HealthSnapshot { status: HealthStatus::Degraded { reason: "lagged" } });
                        }
                    }
                }
            }
        });

        self.handles.lock().await.push(h);
    }

    pub async fn shutdown(self: Arc<Self>) {
        let _ = self.stop_tx.send(true);
        let mut hs = self.handles.lock().await;
        for h in hs.drain(..) {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(2), h).await;
        }
    }
}
