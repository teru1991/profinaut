use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::watch;

use crate::{SymbolAdapterError, SymbolContext, SymbolEventStream, SymbolSubscriber};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResyncHint {
    Lagged { reason: &'static str },
    Reset { reason: &'static str },
}

#[derive(Clone)]
pub struct ResyncSignal {
    tx: watch::Sender<Option<ResyncHint>>,
    rx: watch::Receiver<Option<ResyncHint>>,
}

impl ResyncSignal {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(None);
        Self { tx, rx }
    }

    pub fn receiver(&self) -> watch::Receiver<Option<ResyncHint>> {
        self.rx.clone()
    }

    pub fn notify(&self, hint: ResyncHint) {
        let _ = self.tx.send(Some(hint));
    }

    pub fn clear(&self) {
        let _ = self.tx.send(None);
    }
}

impl Default for ResyncSignal {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    type Snapshot: Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn fetch_snapshot(&self) -> Result<Self::Snapshot, Self::Error>;
}

#[async_trait]
pub trait SymbolSubscriberExtResync: SymbolSubscriber {
    fn resync_receiver(&self) -> Option<watch::Receiver<Option<ResyncHint>>> {
        None
    }
}

pub async fn subscribe_with_optional_resync<S>(
    sub: Arc<S>,
    ctx: &SymbolContext,
) -> Result<
    (
        SymbolEventStream,
        Option<watch::Receiver<Option<ResyncHint>>>,
    ),
    SymbolAdapterError,
>
where
    S: SymbolSubscriber + SymbolSubscriberExtResync + 'static,
{
    let stream = sub.subscribe_events(ctx).await?;
    Ok((stream, sub.resync_receiver()))
}
