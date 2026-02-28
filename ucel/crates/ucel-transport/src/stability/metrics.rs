use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct StabilityMetrics {
    counters: Mutex<BTreeMap<String, u64>>,
    gauges: Mutex<BTreeMap<String, i64>>,
}

impl StabilityMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_counter(&self, name: &str, label: &str, by: u64) {
        let mut g = self.counters.lock().expect("metrics counters poisoned");
        let key = format!("{}{{{}}}", name, label);
        *g.entry(key).or_insert(0) += by;
    }

    pub fn add_gauge(&self, name: &str, by: i64) {
        let mut g = self.gauges.lock().expect("metrics gauges poisoned");
        *g.entry(name.to_string()).or_insert(0) += by;
    }

    pub fn set_gauge(&self, name: &str, value: i64) {
        let mut g = self.gauges.lock().expect("metrics gauges poisoned");
        g.insert(name.to_string(), value);
    }
}
