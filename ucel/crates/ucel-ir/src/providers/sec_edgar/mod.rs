pub mod ticker_cik;

use crate::checkpoint::CheckpointStore;
use crate::domain::{
    ArtifactKind, ArtifactRef, CanonicalEntityId, EntityAlias, IrEvent, IrProvider, Quality,
    QualityStatus,
};
use crate::errors::{UcelIrError, UcelIrErrorKind};
use crate::sinks::RawSink;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use ticker_cik::TickerCikCache;

#[derive(Debug, Clone)]
pub struct SecEdgarConfig {
    pub user_agent: String,
    pub fixtures_dir: PathBuf,
    pub tickers: Vec<String>,
    pub ciks: Vec<String>,
    pub max_rps: u32,
    pub burst: u32,
}

impl SecEdgarConfig {
    pub fn with_defaults(user_agent: impl Into<String>, fixtures_dir: PathBuf) -> Self {
        Self {
            user_agent: user_agent.into(),
            fixtures_dir,
            tickers: Vec::new(),
            ciks: Vec::new(),
            max_rps: 5,
            burst: 10,
        }
    }

    pub fn validate(&self) -> Result<(), UcelIrError> {
        if self.user_agent.trim().is_empty() {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "sec_edgar.user_agent is required",
            ));
        }
        if self.max_rps == 0 || self.burst == 0 {
            return Err(UcelIrError::new(
                UcelIrErrorKind::Config,
                "sec_edgar.max_rps and sec_edgar.burst must be > 0",
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SecEdgarProvider {
    config: SecEdgarConfig,
    ticker_cache: TickerCikCache,
}

#[derive(Debug, Clone)]
pub struct SecListEventsRequest;

#[derive(Debug, Clone)]
pub struct SecListEventsResponse {
    pub events: Vec<IrEvent>,
}

#[derive(Debug, Clone)]
pub struct SecFetchArtifactRequest {
    pub cik: String,
    pub accession: String,
    pub key: String,
}

impl SecEdgarProvider {
    pub fn new(
        config: SecEdgarConfig,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<Self, UcelIrError> {
        config.validate()?;
        let ticker_cache = TickerCikCache::from_fixture(
            &config.fixtures_dir.join("ticker_cik.json"),
            checkpoints,
        )?;
        Ok(Self {
            config,
            ticker_cache,
        })
    }

    pub fn list_events(
        &self,
        _request: &SecListEventsRequest,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<SecListEventsResponse, UcelIrError> {
        let mut ciks = self.config.ciks.clone();
        for ticker in &self.config.tickers {
            if let Some(cik) = self.ticker_cache.lookup(ticker) {
                ciks.push(cik);
            }
        }
        ciks.sort();
        ciks.dedup();

        let mut events = Vec::new();
        for cik in ciks {
            let parsed = self.load_submissions(&cik)?;
            let last_key = format!("sec:last_accession:{cik}");
            let last_seen = checkpoints.get(&last_key)?;
            let mut started = last_seen.is_none();
            let mut new_last: Option<String> = None;

            for f in parsed.filings.recent {
                if !started {
                    if Some(f.accession_number.as_str()) == last_seen.as_deref() {
                        started = true;
                    }
                    continue;
                }

                if new_last.is_none() {
                    new_last = Some(f.accession_number.clone());
                }

                events.push(IrEvent {
                    provider: IrProvider::SecEdgar,
                    source_event_id: f.accession_number.clone(),
                    entity_id: CanonicalEntityId::Cik(cik.clone()),
                    entity_aliases: vec![EntityAlias {
                        namespace: "US:TICKER".to_string(),
                        value: parsed.ticker.clone(),
                    }],
                    filing_type: f.form,
                    filing_date: Some(f.filing_date),
                    published_at: None,
                    observed_at: now_unix_secs(),
                    artifacts: vec![],
                    quality: Quality {
                        status: QualityStatus::Ok,
                        missing: Vec::new(),
                        anomaly_flags: Vec::new(),
                        http_status: None,
                        confidence: 1.0,
                    },
                    trace_id: format!("sec:{cik}:{}", f.accession_number),
                });
            }

            if let Some(last) = new_last {
                checkpoints.set(&last_key, &last)?;
            }
        }

        Ok(SecListEventsResponse { events })
    }

    pub fn fetch_artifact(
        &self,
        request: &SecFetchArtifactRequest,
        raw_sink: &dyn RawSink,
    ) -> Result<ArtifactRef, UcelIrError> {
        let path = self.config.fixtures_dir.join(format!(
            "artifact_{}.html",
            request.accession.replace('-', "")
        ));
        let bytes = fs::read(path)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;
        raw_sink.put_raw(&request.key, &bytes)?;

        Ok(ArtifactRef {
            kind: ArtifactKind::FilingDocument,
            uri: format!("raw://{}", request.key),
            source_url: format!("sec://{}/{}", request.cik, request.accession),
            sha256: Some(hex::encode(Sha256::digest(&bytes))),
            content_length: Some(bytes.len() as u64),
            mime: Some("text/html".to_string()),
            etag: None,
            last_modified: None,
            retrieved_at: Some(now_unix_secs()),
        })
    }

    fn load_submissions(&self, cik: &str) -> Result<Submissions, UcelIrError> {
        let path = self
            .config
            .fixtures_dir
            .join(format!("submissions_CIK{}.json", cik));
        let body = fs::read_to_string(path)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))?;
        serde_json::from_str(&body)
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()))
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Deserialize)]
struct Submissions {
    ticker: String,
    filings: Filings,
}

#[derive(Debug, Deserialize)]
struct Filings {
    recent: Vec<RecentFiling>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentFiling {
    accession_number: String,
    filing_date: String,
    form: String,
}
