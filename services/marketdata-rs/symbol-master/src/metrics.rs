use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Metrics {
    counters: BTreeMap<String, u64>,
}

impl Metrics {
    pub fn inc(&mut self, key: impl Into<String>) {
        *self.counters.entry(key.into()).or_insert(0) += 1;
    }

    pub fn get(&self, key: &str) -> u64 {
        *self.counters.get(key).unwrap_or(&0)
    }
}
