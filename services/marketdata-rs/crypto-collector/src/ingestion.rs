use crate::envelope::Envelope;
use crate::metrics::Metrics;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{self, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPolicy {
    TradeNoDrop,
    TickerDropOldKeepLatest,
    DepthDropOldDeltasBestEffort,
}

impl ChannelPolicy {
    pub fn for_channel(channel: &str) -> Self {
        match channel {
            "trade" => Self::TradeNoDrop,
            "ticker" => Self::TickerDropOldKeepLatest,
            "depth" => Self::DepthDropOldDeltasBestEffort,
            _ => Self::TradeNoDrop,
        }
    }
}

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("trade overflow for exchange={exchange}")]
    TradeOverflow { exchange: String },
    #[error("ingest channel closed")]
    Closed,
}

pub trait Sink: Send {
    fn emit_batch(&mut self, batch: Vec<Envelope>) -> Result<(), SinkError>;
}

#[derive(Debug, Clone)]
pub struct SinkError {
    pub message: String,
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SinkError {}

enum ControlMessage {
    Flush(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct IngestSender {
    tx: mpsc::Sender<Envelope>,
    metrics: Arc<Metrics>,
}

impl IngestSender {
    pub fn try_send(&self, envelope: Envelope) -> Result<(), IngestError> {
        let exchange = envelope.exchange.clone();
        let channel = envelope.channel.clone();
        self.metrics.inc_ingest_messages_total(&exchange, &channel);

        match self.tx.try_send(envelope) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => match ChannelPolicy::for_channel(&channel) {
                ChannelPolicy::TradeNoDrop => {
                    self.metrics.inc_trade_overflow_total(&exchange);
                    self.metrics.inc_ingest_errors_total(&exchange);
                    Err(IngestError::TradeOverflow { exchange })
                }
                ChannelPolicy::TickerDropOldKeepLatest
                | ChannelPolicy::DepthDropOldDeltasBestEffort => {
                    self.metrics.inc_drop_count(&exchange, &channel);
                    Ok(())
                }
            },
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.metrics.inc_ingest_errors_total(&exchange);
                Err(IngestError::Closed)
            }
        }
    }
}

pub struct PipelineHandle {
    control_tx: mpsc::UnboundedSender<ControlMessage>,
    join: tokio::task::JoinHandle<()>,
}

impl PipelineHandle {
    pub async fn flush(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.control_tx.send(ControlMessage::Flush(tx));
        let _ = rx.await;
    }

    pub async fn shutdown(self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.control_tx.send(ControlMessage::Shutdown(tx));
        let _ = rx.await;
        let _ = self.join.await;
    }
}

pub struct BufferRunner;

impl BufferRunner {
    pub fn spawn(
        capacity: usize,
        max_batch_items: usize,
        max_batch_interval_ms: u64,
        sink: Box<dyn Sink>,
        metrics: Arc<Metrics>,
    ) -> (IngestSender, PipelineHandle) {
        let (tx, mut rx) = mpsc::channel::<Envelope>(capacity);
        let (control_tx, mut control_rx) = mpsc::unbounded_channel();
        let sink = Arc::new(Mutex::new(sink));

        let runner_metrics = metrics.clone();
        let join = tokio::spawn(async move {
            let mut buffer: Vec<Envelope> = Vec::with_capacity(max_batch_items.max(1));
            let mut interval = time::interval(Duration::from_millis(max_batch_interval_ms.max(1)));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            flush_batch(&mut buffer, sink.clone(), runner_metrics.clone()).await;
                        }
                    }
                    Some(ctrl) = control_rx.recv() => {
                        match ctrl {
                            ControlMessage::Flush(done) => {
                                while let Ok(env) = rx.try_recv() {
                                    buffer.push(env);
                                }
                                flush_batch(&mut buffer, sink.clone(), runner_metrics.clone()).await;
                                let _ = done.send(());
                            }
                            ControlMessage::Shutdown(done) => {
                                while let Ok(env) = rx.try_recv() {
                                    buffer.push(env);
                                }
                                flush_batch(&mut buffer, sink.clone(), runner_metrics.clone()).await;
                                let _ = done.send(());
                                break;
                            }
                        }
                    }
                    msg = rx.recv() => {
                        match msg {
                            Some(env) => {
                                let exchange = env.exchange.clone();
                                buffer.push(env);
                                runner_metrics.set_buffer_depth(&exchange, buffer.len());
                                if buffer.len() >= max_batch_items {
                                    flush_batch(&mut buffer, sink.clone(), runner_metrics.clone()).await;
                                }
                            }
                            None => {
                                flush_batch(&mut buffer, sink.clone(), runner_metrics.clone()).await;
                                break;
                            }
                        }
                    }
                }
            }
        });

        (
            IngestSender {
                tx,
                metrics: metrics.clone(),
            },
            PipelineHandle { control_tx, join },
        )
    }
}

async fn flush_batch(
    buffer: &mut Vec<Envelope>,
    sink: Arc<Mutex<Box<dyn Sink>>>,
    metrics: Arc<Metrics>,
) {
    if buffer.is_empty() {
        return;
    }

    let exchange = buffer[0].exchange.clone();
    let batch = std::mem::take(buffer);
    let mut locked_sink = sink.lock().await;
    if locked_sink.emit_batch(batch).is_err() {
        metrics.inc_ingest_errors_total(&exchange);
    }
    metrics.set_buffer_depth(&exchange, 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Envelope;
    use serde_json::json;
    use std::sync::Mutex as StdMutex;
    use tokio::time::sleep;

    #[derive(Default)]
    struct MemorySink {
        batches: Arc<StdMutex<Vec<Vec<Envelope>>>>,
    }

    impl MemorySink {
        fn new_with_shared() -> (Self, Arc<StdMutex<Vec<Vec<Envelope>>>>) {
            let shared = Arc::new(StdMutex::new(Vec::new()));
            (
                Self {
                    batches: shared.clone(),
                },
                shared,
            )
        }
    }

    impl Sink for MemorySink {
        fn emit_batch(&mut self, batch: Vec<Envelope>) -> Result<(), SinkError> {
            self.batches.lock().expect("poisoned").push(batch);
            Ok(())
        }
    }

    fn env(channel: &str) -> Envelope {
        Envelope::builder(
            "adapter@1",
            "cid",
            "binance-main",
            "BTCUSDT",
            channel,
            json!({"k":1}),
        )
        .build()
    }

    #[tokio::test]
    async fn batching_by_count_threshold() {
        let metrics = Arc::new(Metrics::default());
        let (sink, batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(16, 2, 1_000, Box::new(sink), metrics);

        sender.try_send(env("trade")).unwrap();
        sender.try_send(env("trade")).unwrap();
        sleep(Duration::from_millis(30)).await;

        let got = batches.lock().expect("poisoned").clone();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].len(), 2);
        handle.shutdown().await;
    }

    #[tokio::test]
    async fn batching_by_time_interval() {
        let metrics = Arc::new(Metrics::default());
        let (sink, batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(16, 50, 30, Box::new(sink), metrics);

        sender.try_send(env("trade")).unwrap();
        sleep(Duration::from_millis(80)).await;

        let got = batches.lock().expect("poisoned").clone();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].len(), 1);
        handle.shutdown().await;
    }

    #[tokio::test]
    async fn flush_drains_remaining_items() {
        let metrics = Arc::new(Metrics::default());
        let (sink, batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(16, 50, 1_000, Box::new(sink), metrics);

        sender.try_send(env("trade")).unwrap();
        handle.flush().await;

        let got = batches.lock().expect("poisoned").clone();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].len(), 1);
        handle.shutdown().await;
    }

    #[tokio::test]
    async fn ticker_drop_increments_drop_count() {
        let metrics = Arc::new(Metrics::default());
        let (sink, _batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(1, 50, 10_000, Box::new(sink), metrics.clone());

        sender.try_send(env("ticker")).unwrap();
        sender.try_send(env("ticker")).unwrap();

        assert_eq!(metrics.drop_count("binance-main", "ticker"), 1);
        handle.shutdown().await;
    }

    #[tokio::test]
    async fn trade_overflow_increments_metric() {
        let metrics = Arc::new(Metrics::default());
        let (sink, _batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(1, 50, 10_000, Box::new(sink), metrics.clone());

        sender.try_send(env("trade")).unwrap();
        let err = sender.try_send(env("trade")).unwrap_err();
        assert!(matches!(err, IngestError::TradeOverflow { .. }));
        assert_eq!(metrics.trade_overflow_total("binance-main"), 1);

        handle.shutdown().await;
    }

    #[tokio::test]
    async fn ingest_messages_metric_increments() {
        let metrics = Arc::new(Metrics::default());
        let (sink, _batches) = MemorySink::new_with_shared();
        let (sender, handle) = BufferRunner::spawn(16, 10, 1_000, Box::new(sink), metrics.clone());

        sender.try_send(env("trade")).unwrap();
        sender.try_send(env("trade")).unwrap();

        assert_eq!(metrics.ingest_messages_total("binance-main", "trade"), 2);
        handle.shutdown().await;
    }
}
