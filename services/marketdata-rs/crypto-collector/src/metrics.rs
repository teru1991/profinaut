use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ExchangeChannelKey {
    exchange: String,
    channel: String,
}

#[derive(Debug, Default)]
pub struct Metrics {
    ingest_errors_total: Mutex<HashMap<String, Arc<AtomicU64>>>,
    ingest_messages_total: Mutex<HashMap<ExchangeChannelKey, Arc<AtomicU64>>>,
    drop_count: Mutex<HashMap<ExchangeChannelKey, Arc<AtomicU64>>>,
    trade_overflow_total: Mutex<HashMap<String, Arc<AtomicU64>>>,
    buffer_depth: Mutex<HashMap<String, Arc<AtomicI64>>>,
}

impl Metrics {
    pub fn inc_ingest_messages_total(&self, exchange: &str, channel: &str) {
        let counter = {
            let mut map = self.ingest_messages_total.lock().expect("poisoned");
            map.entry(ExchangeChannelKey {
                exchange: exchange.to_string(),
                channel: channel.to_string(),
            })
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_ingest_errors_total(&self, exchange: &str) {
        let counter = {
            let mut map = self.ingest_errors_total.lock().expect("poisoned");
            map.entry(exchange.to_string())
                .or_insert_with(|| Arc::new(AtomicU64::new(0)))
                .clone()
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_drop_count(&self, exchange: &str, channel: &str) {
        let counter = {
            let mut map = self.drop_count.lock().expect("poisoned");
            map.entry(ExchangeChannelKey {
                exchange: exchange.to_string(),
                channel: channel.to_string(),
            })
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_trade_overflow_total(&self, exchange: &str) {
        let counter = {
            let mut map = self.trade_overflow_total.lock().expect("poisoned");
            map.entry(exchange.to_string())
                .or_insert_with(|| Arc::new(AtomicU64::new(0)))
                .clone()
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_buffer_depth(&self, exchange: &str, depth: usize) {
        let gauge = {
            let mut map = self.buffer_depth.lock().expect("poisoned");
            map.entry(exchange.to_string())
                .or_insert_with(|| Arc::new(AtomicI64::new(0)))
                .clone()
        };
        gauge.store(depth as i64, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub fn ingest_messages_total(&self, exchange: &str, channel: &str) -> u64 {
        let map = self.ingest_messages_total.lock().expect("poisoned");
        map.get(&ExchangeChannelKey {
            exchange: exchange.to_string(),
            channel: channel.to_string(),
        })
        .map(|v| v.load(Ordering::Relaxed))
        .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn ingest_errors_total(&self, exchange: &str) -> u64 {
        let map = self.ingest_errors_total.lock().expect("poisoned");
        map.get(exchange)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn drop_count(&self, exchange: &str, channel: &str) -> u64 {
        let map = self.drop_count.lock().expect("poisoned");
        map.get(&ExchangeChannelKey {
            exchange: exchange.to_string(),
            channel: channel.to_string(),
        })
        .map(|v| v.load(Ordering::Relaxed))
        .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn trade_overflow_total(&self, exchange: &str) -> u64 {
        let map = self.trade_overflow_total.lock().expect("poisoned");
        map.get(exchange)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}
