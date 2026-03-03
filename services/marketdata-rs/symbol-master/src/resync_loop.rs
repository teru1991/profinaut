use crate::snapshot::{fetch_snapshot, SnapshotError};
use crate::store_bridge::{apply_snapshot_to_store, record_checkpoint_jsonl};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{watch, Mutex};
use ucel_symbol_adapter::ResyncHint;
use ucel_symbol_store::SymbolStore;

#[derive(Clone, Debug, Default)]
pub struct ResyncCoordinatorState {
    pub last_error: Option<&'static str>,
    pub last_hint: Option<&'static str>,
    pub last_store_version: Option<u64>,
}

type ExchangeId = String;

pub struct ResyncCoordinator {
    inner: Mutex<Inner>,
}

struct ExchangeResyncInput {
    receiver: watch::Receiver<Option<ResyncHint>>,
    snapshot_url: Option<String>,
}

struct Inner {
    exchanges: HashMap<ExchangeId, ExchangeResyncInput>,
    state: ResyncCoordinatorState,
    checkpoint_path: PathBuf,
    store: Arc<SymbolStore>,
}

impl Default for ResyncCoordinator {
    fn default() -> Self {
        Self::new(
            Arc::new(SymbolStore::new()),
            PathBuf::from("services/marketdata-rs/symbol-master/checkpoints.jsonl"),
        )
    }
}

impl ResyncCoordinator {
    pub fn new(store: Arc<SymbolStore>, checkpoint_path: PathBuf) -> Self {
        Self {
            inner: Mutex::new(Inner {
                exchanges: HashMap::new(),
                state: ResyncCoordinatorState::default(),
                checkpoint_path,
                store,
            }),
        }
    }

    pub async fn register_exchange(
        &self,
        exchange_id: impl Into<String>,
        rx: watch::Receiver<Option<ResyncHint>>,
        snapshot_url: Option<String>,
    ) {
        let mut g = self.inner.lock().await;
        g.exchanges.insert(
            exchange_id.into(),
            ExchangeResyncInput {
                receiver: rx,
                snapshot_url,
            },
        );
    }

    pub async fn snapshot(&self) -> ResyncCoordinatorState {
        self.inner.lock().await.state.clone()
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            let mut to_resync: Vec<(String, &'static str, Option<String>)> = Vec::new();
            {
                let mut g = self.inner.lock().await;
                let mut latest_hint: Option<&'static str> = None;
                for (exchange_id, entry) in &mut g.exchanges {
                    if entry.receiver.has_changed().unwrap_or(false) {
                        let hint = entry.receiver.borrow_and_update().clone();
                        if let Some(h) = hint {
                            let hint_label = match h {
                                ResyncHint::Lagged { .. } => "lagged",
                                ResyncHint::Reset { .. } => "reset",
                            };
                            latest_hint = Some(hint_label);
                            to_resync.push((
                                exchange_id.clone(),
                                hint_label,
                                entry.snapshot_url.clone(),
                            ));
                        }
                    }
                }
                if latest_hint.is_some() {
                    g.state.last_hint = latest_hint;
                }
            }

            for (exchange_id, _hint, snapshot_url) in to_resync {
                let Some(url) = snapshot_url else {
                    self.set_error("snapshot_url_missing").await;
                    continue;
                };

                match fetch_snapshot(&exchange_id, &url).await {
                    Ok(raw) => {
                        let (store, checkpoint_path) = {
                            let g = self.inner.lock().await;
                            (g.store.clone(), g.checkpoint_path.clone())
                        };
                        match apply_snapshot_to_store(&store, &raw.exchange_id, &raw.body) {
                            Ok(cp) => {
                                if record_checkpoint_jsonl(&checkpoint_path, &raw.exchange_id, &cp)
                                    .is_err()
                                {
                                    self.set_error("checkpoint_write_failed").await;
                                } else {
                                    {
                                        let mut g = self.inner.lock().await;
                                        g.state.last_store_version = Some(cp.store_version);
                                    }
                                    self.clear_error().await;
                                }
                            }
                            Err(_) => self.set_error("store_apply_failed").await,
                        }
                    }
                    Err(err) => match err {
                        SnapshotError::MissingUrl => self.set_error("snapshot_url_missing").await,
                        SnapshotError::Http(_) => self.set_error("snapshot_http_failed").await,
                        SnapshotError::Json(_) => self.set_error("snapshot_json_failed").await,
                    },
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
