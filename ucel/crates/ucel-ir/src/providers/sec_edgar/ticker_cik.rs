use crate::checkpoint::CheckpointStore;
use crate::errors::{UcelIrError, UcelIrErrorKind};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct TickerCikCache {
    map: HashMap<String, String>,
}

impl TickerCikCache {
    pub fn from_fixture(
        path: &Path,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<Self, UcelIrError> {
        let body = fs::read_to_string(path)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;
        let rows: Vec<TickerCikRow> = serde_json::from_str(&body)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;

        let mut map = HashMap::new();
        for row in rows {
            map.insert(row.ticker.to_uppercase(), row.cik_str.clone());
            checkpoints.set(
                &format!("sec:ticker_cik:{}", row.ticker.to_uppercase()),
                &row.cik_str,
            )?;
        }

        Ok(Self { map })
    }

    pub fn lookup(&self, ticker: &str) -> Option<String> {
        self.map.get(&ticker.to_uppercase()).cloned()
    }
}

#[derive(Debug, Deserialize)]
struct TickerCikRow {
    ticker: String,
    cik_str: String,
}
