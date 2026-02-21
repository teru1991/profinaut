use crate::checkpoint::CheckpointStore;
use crate::config::UcelIrConfig;
use crate::domain::{CanonicalEntityId, IrEvent, QualityStatus};
use crate::errors::{UcelIrError, UcelIrErrorKind};
use crate::http::HttpClient;
use crate::providers::edinet::{
    EdinetConfig, EdinetProvider, FetchArtifactRequest, IrProviderSource, ListEventsRequest,
};
use crate::providers::sec_edgar::{
    SecEdgarConfig, SecEdgarProvider, SecFetchArtifactRequest, SecListEventsRequest,
};
use crate::sinks::{EventSink, RawSink};
use std::collections::BTreeMap;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProviderSyncStats {
    pub detected: usize,
    pub saved: usize,
    pub deduplicated: usize,
    pub degraded: usize,
    pub retries: usize,
    pub status_429: usize,
    pub parse_errors: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub providers: BTreeMap<String, ProviderSyncStats>,
    pub errors: Vec<String>,
    pub emitted_events: Vec<IrEvent>,
}

#[derive(Debug, Clone)]
pub struct SyncRequest {
    pub edinet: Option<EdinetSyncConfig>,
    pub sec_edgar: Option<SecEdgarSyncConfig>,
    pub fetch_artifacts: bool,
}

#[derive(Debug, Clone)]
pub struct EdinetSyncConfig {
    pub date: String,
    pub config: EdinetConfig,
}

#[derive(Debug, Clone)]
pub struct SecEdgarSyncConfig {
    pub config: SecEdgarConfig,
}

pub type IrEventStream = mpsc::Receiver<IrEvent>;

pub struct UcelIrClient {
    pub config: UcelIrConfig,
    pub http: HttpClient,
}

impl UcelIrClient {
    pub fn new(config: UcelIrConfig) -> Result<Self, UcelIrError> {
        config.validate()?;
        let http = HttpClient::new(config.http.clone())?;
        Ok(Self { config, http })
    }

    pub fn sync_events(
        &self,
        events: Vec<IrEvent>,
        event_sink: &dyn EventSink,
        checkpoint_store: Option<Arc<dyn CheckpointStore + Send + Sync>>,
    ) -> Result<ProviderSyncStats, UcelIrError> {
        let mut stats = ProviderSyncStats {
            detected: events.len(),
            ..ProviderSyncStats::default()
        };

        for event in events {
            if event_sink.put_event(event)? {
                stats.saved += 1;
            } else {
                stats.deduplicated += 1;
            }
        }

        if let Some(store) = checkpoint_store {
            store.set("last_sync_status", "ok")?;
        }

        Ok(stats)
    }

    pub fn sync_once(
        &self,
        request: &SyncRequest,
        event_sink: &dyn EventSink,
        raw_sink: &dyn RawSink,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<SyncReport, UcelIrError> {
        let mut report = SyncReport::default();

        if let Some(edinet) = &request.edinet {
            let mut stats = ProviderSyncStats::default();
            let provider = EdinetProvider::new(
                HttpClient::new(self.config.http.clone())?,
                edinet.config.clone(),
            );
            match provider.list_events(
                &ListEventsRequest {
                    date: edinet.date.clone(),
                },
                checkpoints,
            ) {
                Ok(resp) => {
                    if resp.degraded {
                        stats.degraded += 1;
                    }
                    for event in resp.events {
                        stats.detected += 1;
                        if event.quality.status != QualityStatus::Ok {
                            stats.degraded += 1;
                        }
                        if event
                            .quality
                            .anomaly_flags
                            .iter()
                            .any(|f| f == "parser_failed")
                        {
                            stats.parse_errors += 1;
                        }
                        if request.fetch_artifacts {
                            let _ = provider.fetch_artifact(
                                &FetchArtifactRequest {
                                    date: edinet.date.clone(),
                                    doc_id: event.source_event_id.clone(),
                                    key: format!(
                                        "edinet/{}/{}",
                                        edinet.date, event.source_event_id
                                    ),
                                },
                                raw_sink,
                            );
                        }
                        if event_sink.put_event(event.clone())? {
                            stats.saved += 1;
                            report.emitted_events.push(event);
                        } else {
                            stats.deduplicated += 1;
                        }
                    }
                    for w in resp.warnings {
                        if w.contains("429") {
                            stats.status_429 += 1;
                        }
                    }
                }
                Err(err) => {
                    if err.kind == UcelIrErrorKind::RateLimit {
                        stats.status_429 += 1;
                    }
                    report.errors.push(format!("edinet: {}", err));
                    stats.degraded += 1;
                }
            }
            report.providers.insert("edinet".to_string(), stats);
        }

        if let Some(sec) = &request.sec_edgar {
            let mut stats = ProviderSyncStats::default();
            match SecEdgarProvider::new(sec.config.clone(), checkpoints) {
                Ok(provider) => match provider.list_events(&SecListEventsRequest, checkpoints) {
                    Ok(resp) => {
                        for event in resp.events {
                            stats.detected += 1;
                            if request.fetch_artifacts {
                                let cik = match &event.entity_id {
                                    CanonicalEntityId::Cik(v) => v.clone(),
                                    _ => "UNKNOWN".to_string(),
                                };
                                let _ = provider.fetch_artifact(
                                    &SecFetchArtifactRequest {
                                        cik,
                                        accession: event.source_event_id.clone(),
                                        key: format!("sec/{}/primary", event.source_event_id),
                                    },
                                    raw_sink,
                                );
                            }
                            if event_sink.put_event(event.clone())? {
                                stats.saved += 1;
                                report.emitted_events.push(event);
                            } else {
                                stats.deduplicated += 1;
                            }
                        }
                    }
                    Err(err) => {
                        if err.kind == UcelIrErrorKind::RateLimit {
                            stats.status_429 += 1;
                        }
                        report.errors.push(format!("sec_edgar: {}", err));
                        stats.degraded += 1;
                    }
                },
                Err(err) => {
                    report.errors.push(format!("sec_edgar: {}", err));
                    stats.degraded += 1;
                }
            }
            report.providers.insert("sec_edgar".to_string(), stats);
        }

        Ok(report)
    }

    pub fn sync_range(
        &self,
        request: &SyncRequest,
        start_date: &str,
        end_date: &str,
        event_sink: &dyn EventSink,
        raw_sink: &dyn RawSink,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<SyncReport, UcelIrError> {
        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Config, e.to_string()))?;
        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Config, e.to_string()))?;

        let mut combined = SyncReport::default();
        let mut day = start;
        while day <= end {
            let mut req = request.clone();
            if let Some(edinet) = &mut req.edinet {
                edinet.date = day.format("%Y-%m-%d").to_string();
            }
            let one = self.sync_once(&req, event_sink, raw_sink, checkpoints)?;
            merge_report(&mut combined, one);
            day = day
                .succ_opt()
                .ok_or_else(|| UcelIrError::new(UcelIrErrorKind::Internal, "date overflow"))?;
        }

        Ok(combined)
    }

    pub fn stream(
        &self,
        request: SyncRequest,
        polls: usize,
        interval: Duration,
        event_sink: Arc<dyn EventSink + Send + Sync>,
        raw_sink: Arc<dyn RawSink + Send + Sync>,
        checkpoints: Arc<dyn CheckpointStore + Send + Sync>,
    ) -> IrEventStream {
        let (tx, rx) = mpsc::channel();
        let client = Self::new(self.config.clone()).expect("stream client config must be valid");

        thread::spawn(move || {
            for _ in 0..polls {
                if let Ok(report) = client.sync_once(
                    &request,
                    event_sink.as_ref(),
                    raw_sink.as_ref(),
                    checkpoints.as_ref(),
                ) {
                    for event in report.emitted_events {
                        let _ = tx.send(event);
                    }
                }
                thread::sleep(interval);
            }
        });

        rx
    }
}

fn merge_report(target: &mut SyncReport, next: SyncReport) {
    target.errors.extend(next.errors);
    target.emitted_events.extend(next.emitted_events);
    for (provider, stats) in next.providers {
        let entry = target.providers.entry(provider).or_default();
        entry.detected += stats.detected;
        entry.saved += stats.saved;
        entry.deduplicated += stats.deduplicated;
        entry.degraded += stats.degraded;
        entry.retries += stats.retries;
        entry.status_429 += stats.status_429;
        entry.parse_errors += stats.parse_errors;
    }
}
