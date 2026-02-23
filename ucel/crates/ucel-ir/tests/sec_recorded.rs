use std::path::PathBuf;

use ucel_ir::{
    MemoryCheckpointStore, MemorySink, SecEdgarConfig, SecEdgarProvider, SecFetchArtifactRequest,
    SecListEventsRequest, UcelIrClient, UcelIrConfig, UcelIrErrorKind,
};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/sec")
}

#[test]
fn list_events_is_stable_from_recorded_submissions() {
    let checkpoints = MemoryCheckpointStore::default();
    let mut config = SecEdgarConfig::with_defaults("ucel-ir-sec-tests/0.1", fixture_dir());
    config.tickers = vec!["AAPL".to_string()];

    let provider = SecEdgarProvider::new(config, &checkpoints).expect("provider init");
    let response = provider
        .list_events(&SecListEventsRequest, &checkpoints)
        .expect("list events");

    assert_eq!(response.events.len(), 2);
    assert_eq!(response.events[0].source_event_id, "0000320193-24-000123");
    assert_eq!(response.events[0].filing_type, "10-K");
}

#[test]
fn sec_events_are_idempotent_by_accession() {
    let checkpoints = MemoryCheckpointStore::default();
    let mut config = SecEdgarConfig::with_defaults("ucel-ir-sec-tests/0.1", fixture_dir());
    config.tickers = vec!["AAPL".to_string()];
    let provider = SecEdgarProvider::new(config, &checkpoints).expect("provider init");

    let sink = MemorySink::default();
    let events = provider
        .list_events(&SecListEventsRequest, &checkpoints)
        .expect("list")
        .events;

    let client = UcelIrClient::new(UcelIrConfig {
        http: ucel_ir::HttpConfig {
            user_agent: "ucel-ir-test/0.1".to_string(),
            timeout_ms: 1_000,
            max_retries: 1,
            base_backoff_ms: 10,
            rate_limit_per_sec: 10,
        },
        raw_storage_root: "/tmp/ucel-ir/raw".to_string(),
        checkpoint_path: "/tmp/ucel-ir/checkpoint".to_string(),
    })
    .expect("client");

    let first = client
        .sync_events(events.clone(), &sink, None)
        .expect("sync1");
    let second = client.sync_events(events, &sink, None).expect("sync2");
    assert_eq!(first.saved, 2);
    assert_eq!(second.deduplicated, 2);
}

#[test]
fn missing_user_agent_returns_config_error() {
    let checkpoints = MemoryCheckpointStore::default();
    let config = SecEdgarConfig::with_defaults("", fixture_dir());
    let err = SecEdgarProvider::new(config, &checkpoints).expect_err("must fail config");
    assert_eq!(err.kind, UcelIrErrorKind::Config);
}

#[test]
fn artifact_save_populates_required_fields() {
    let checkpoints = MemoryCheckpointStore::default();
    let mut config = SecEdgarConfig::with_defaults("ucel-ir-sec-tests/0.1", fixture_dir());
    config.ciks = vec!["0000320193".to_string()];
    let provider = SecEdgarProvider::new(config, &checkpoints).expect("provider init");
    let sink = MemorySink::default();

    let artifact = provider
        .fetch_artifact(
            &SecFetchArtifactRequest {
                cik: "0000320193".to_string(),
                accession: "0000320193-24-000123".to_string(),
                key: "sec/0000320193/0000320193-24-000123/primary".to_string(),
            },
            &sink,
        )
        .expect("artifact");

    assert_eq!(artifact.mime.as_deref(), Some("text/html"));
    assert!(artifact.content_length.unwrap_or_default() > 0);
    assert!(artifact.sha256.as_ref().is_some_and(|s| !s.is_empty()));
}
