use crate::checkpoint::CheckpointStore;
use crate::domain::{
    ArtifactKind, ArtifactRef, CanonicalEntityId, EntityAlias, IrEvent,
    IrProvider as IrProviderKind, Quality, QualityStatus,
};
use crate::errors::{UcelIrError, UcelIrErrorKind};
use crate::http::HttpClient;
use crate::sinks::RawSink;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_LIST_URL: &str = "https://api.edinet-fsa.go.jp/api/v2/documents.json";

#[derive(Debug, Clone)]
pub struct EdinetConfig {
    pub api_key: Option<String>,
    pub fixtures_dir: Option<PathBuf>,
    pub list_url: String,
}

impl Default for EdinetConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            fixtures_dir: None,
            list_url: DEFAULT_LIST_URL.to_string(),
        }
    }
}

pub struct EdinetProvider {
    http: HttpClient,
    config: EdinetConfig,
}

#[derive(Debug, Clone)]
pub struct ListEventsRequest {
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct ListEventsResponse {
    pub events: Vec<IrEvent>,
    pub degraded: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FetchArtifactRequest {
    pub date: String,
    pub doc_id: String,
    pub key: String,
}

pub trait IrProviderSource {
    fn list_events(
        &self,
        request: &ListEventsRequest,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<ListEventsResponse, UcelIrError>;

    fn fetch_artifact(
        &self,
        request: &FetchArtifactRequest,
        raw_sink: &dyn RawSink,
    ) -> Result<ArtifactRef, UcelIrError>;
}

impl EdinetProvider {
    pub fn new(http: HttpClient, config: EdinetConfig) -> Self {
        Self { http, config }
    }

    fn checkpoint_last_date() -> &'static str {
        "edinet:last_date"
    }

    fn checkpoint_last_seen_doc_id(date: &str) -> String {
        format!("edinet:last_seen_doc_id:{date}")
    }

    fn load_daily_documents(&self, date: &str) -> Result<String, UcelIrError> {
        if let Some(fixtures_dir) = &self.config.fixtures_dir {
            let path = fixtures_dir.join(format!("documents_{date}.json"));
            return fs::read_to_string(path)
                .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()));
        }

        let mut request = self
            .http
            .inner()
            .get(&self.config.list_url)
            .query(&[("date", date), ("type", "2")]);
        if let Some(api_key) = self.config.api_key.as_ref() {
            request = request.header("X-API-KEY", api_key);
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Internal, e.to_string()))?;
        let response = runtime.block_on(async {
            self.http
                .send_with_retry(move |_| request.try_clone().expect("request clone"))
                .await
        })?;

        runtime
            .block_on(async { response.text().await })
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Http, e.to_string()))
    }

    fn load_artifact_bytes(&self, request: &FetchArtifactRequest) -> Result<Vec<u8>, UcelIrError> {
        if let Some(fixtures_dir) = &self.config.fixtures_dir {
            let path = fixtures_dir.join(format!("artifact_{}.bin", request.doc_id));
            return fs::read(path)
                .map_err(|e| UcelIrError::new(UcelIrErrorKind::Upstream, e.to_string()));
        }

        let url = format!(
            "https://api.edinet-fsa.go.jp/api/v2/documents/{}",
            request.doc_id
        );
        let mut request_builder = self.http.inner().get(url).query(&[("type", "1")]);
        if let Some(api_key) = self.config.api_key.as_ref() {
            request_builder = request_builder.header("X-API-KEY", api_key);
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Internal, e.to_string()))?;
        let response = runtime.block_on(async {
            self.http
                .send_with_retry(move |_| request_builder.try_clone().expect("request clone"))
                .await
        })?;

        runtime
            .block_on(async { response.bytes().await })
            .map(|b| b.to_vec())
            .map_err(|e| UcelIrError::new(UcelIrErrorKind::Http, e.to_string()))
    }
}

impl IrProviderSource for EdinetProvider {
    fn list_events(
        &self,
        request: &ListEventsRequest,
        checkpoints: &dyn CheckpointStore,
    ) -> Result<ListEventsResponse, UcelIrError> {
        let mut warnings = Vec::new();
        let missing_key = self.config.api_key.is_none();
        if missing_key {
            warnings.push("missing_api_key".to_string());
        }

        let payload = match self.load_daily_documents(&request.date) {
            Ok(payload) => payload,
            Err(err) if err.is_retryable() || missing_key => {
                warnings.push(format!("degraded: {}", err.message));
                return Ok(ListEventsResponse {
                    events: Vec::new(),
                    degraded: true,
                    warnings,
                });
            }
            Err(err) => return Err(err),
        };

        let parsed: EdinetDocumentList = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(_) => {
                return Ok(ListEventsResponse {
                    events: vec![IrEvent {
                        provider: IrProviderKind::Edinet,
                        source_event_id: format!("parse-error:{}", request.date),
                        entity_id: CanonicalEntityId::EdinetCode("UNKNOWN".to_string()),
                        entity_aliases: vec![],
                        filing_type: "unknown".to_string(),
                        filing_date: Some(request.date.clone()),
                        published_at: None,
                        observed_at: now_unix_secs(),
                        artifacts: vec![],
                        quality: Quality {
                            status: QualityStatus::Degraded,
                            missing: vec!["document_list".to_string()],
                            anomaly_flags: vec!["parser_failed".to_string()],
                            http_status: None,
                            confidence: 0.0,
                        },
                        trace_id: format!("edinet-{}", request.date),
                    }],
                    degraded: true,
                    warnings,
                });
            }
        };

        let last_seen_key = Self::checkpoint_last_seen_doc_id(&request.date);
        let last_seen_doc_id = checkpoints.get(&last_seen_key)?;
        let mut started = last_seen_doc_id.is_none();
        let mut events = Vec::new();
        let mut new_last_seen: Option<String> = None;

        for item in parsed.results {
            if !started {
                if Some(item.doc_id.as_str()) == last_seen_doc_id.as_deref() {
                    started = true;
                }
                continue;
            }

            if new_last_seen.is_none() {
                new_last_seen = Some(item.doc_id.clone());
            }

            let mut quality = Quality::default();
            let entity_id = match item.edinet_code.clone() {
                Some(code) if !code.is_empty() => CanonicalEntityId::EdinetCode(code),
                _ => {
                    quality.status = QualityStatus::Partial;
                    quality.missing.push("edinet_code".to_string());
                    quality.anomaly_flags.push("parser_failed".to_string());
                    CanonicalEntityId::EdinetCode("UNKNOWN".to_string())
                }
            };

            if missing_key {
                quality.status = QualityStatus::Degraded;
                quality.anomaly_flags.push("missing_api_key".to_string());
                quality.confidence = 0.6;
            }

            events.push(IrEvent {
                provider: IrProviderKind::Edinet,
                source_event_id: item.doc_id.clone(),
                entity_id,
                entity_aliases: item
                    .sec_code
                    .map(|v| EntityAlias {
                        namespace: "JP:SEC_CODE".to_string(),
                        value: v,
                    })
                    .into_iter()
                    .collect(),
                filing_type: item
                    .doc_description
                    .unwrap_or_else(|| "unknown".to_string()),
                filing_date: Some(request.date.clone()),
                published_at: None,
                observed_at: now_unix_secs(),
                artifacts: vec![],
                quality,
                trace_id: format!("edinet:{}", item.doc_id),
            });
        }

        checkpoints.set(Self::checkpoint_last_date(), &request.date)?;
        if let Some(last_seen) = new_last_seen {
            checkpoints.set(&last_seen_key, &last_seen)?;
        }

        Ok(ListEventsResponse {
            degraded: missing_key,
            events,
            warnings,
        })
    }

    fn fetch_artifact(
        &self,
        request: &FetchArtifactRequest,
        raw_sink: &dyn RawSink,
    ) -> Result<ArtifactRef, UcelIrError> {
        let bytes = self.load_artifact_bytes(request)?;
        raw_sink.put_raw(&request.key, &bytes)?;

        let sha256 = hex::encode(Sha256::digest(&bytes));
        let mime = if is_zip_like(&bytes) {
            Some("application/zip".to_string())
        } else {
            Some("application/octet-stream".to_string())
        };

        Ok(ArtifactRef {
            kind: ArtifactKind::FilingDocument,
            uri: format!("raw://{}", request.key),
            source_url: format!("edinet://{}/{}", request.date, request.doc_id),
            sha256: Some(sha256),
            content_length: Some(bytes.len() as u64),
            mime,
            etag: None,
            last_modified: None,
            retrieved_at: Some(now_unix_secs()),
        })
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_zip_like(bytes: &[u8]) -> bool {
    bytes.len() > 4 && bytes[0] == 0x50 && bytes[1] == 0x4b
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EdinetDocumentList {
    #[serde(default)]
    results: Vec<EdinetDocument>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EdinetDocument {
    #[serde(rename = "docID")]
    doc_id: String,
    #[serde(default)]
    edinet_code: Option<String>,
    #[serde(default)]
    sec_code: Option<String>,
    #[serde(default)]
    doc_description: Option<String>,
}

pub fn fixture_dir_from(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("testdata/edinet")
}
