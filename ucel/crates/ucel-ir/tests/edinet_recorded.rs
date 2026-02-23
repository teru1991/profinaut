use std::path::PathBuf;

use ucel_ir::{
    EdinetConfig, EdinetProvider, FetchArtifactRequest, HttpConfig, IrProviderSource,
    ListEventsRequest, MemoryCheckpointStore, MemorySink, UcelIrClient, UcelIrConfig,
};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/edinet")
}

fn test_http_config() -> HttpConfig {
    HttpConfig {
        user_agent: "ucel-ir-edinet-tests/0.1".to_string(),
        timeout_ms: 1_000,
        max_retries: 2,
        base_backoff_ms: 5,
        rate_limit_per_sec: 10,
    }
}

#[test]
fn list_events_from_recorded_fixture_is_stable_and_degraded_without_api_key() {
    let provider = EdinetProvider::new(
        ucel_ir::http::HttpClient::new(test_http_config()).expect("http client"),
        EdinetConfig {
            api_key: None,
            fixtures_dir: Some(fixture_dir()),
            list_url: "http://unused".to_string(),
        },
    );
    let checkpoints = MemoryCheckpointStore::default();

    let response = provider
        .list_events(
            &ListEventsRequest {
                date: "2026-01-01".to_string(),
            },
            &checkpoints,
        )
        .expect("list events should succeed");

    assert!(response.degraded);
    assert_eq!(response.events.len(), 2);
    assert_eq!(response.events[0].source_event_id, "S100AAA1");
    assert_eq!(response.events[1].source_event_id, "S100AAA2");
    assert!(response.events[1]
        .quality
        .anomaly_flags
        .contains(&"parser_failed".to_string()));
}

#[test]
fn provider_events_remain_idempotent_when_synced_twice() {
    let provider = EdinetProvider::new(
        ucel_ir::http::HttpClient::new(test_http_config()).expect("http client"),
        EdinetConfig {
            api_key: Some("test-key".to_string()),
            fixtures_dir: Some(fixture_dir()),
            list_url: "http://unused".to_string(),
        },
    );
    let checkpoints = MemoryCheckpointStore::default();
    let sink = MemorySink::default();

    let events_first = provider
        .list_events(
            &ListEventsRequest {
                date: "2026-01-01".to_string(),
            },
            &checkpoints,
        )
        .expect("first list should work")
        .events;

    let client = UcelIrClient::new(UcelIrConfig {
        http: test_http_config(),
        raw_storage_root: "/tmp/ucel-ir/raw".to_string(),
        checkpoint_path: "/tmp/ucel-ir/checkpoint".to_string(),
    })
    .expect("client initialize");

    let first = client
        .sync_events(events_first.clone(), &sink, None)
        .expect("first sync");
    let second = client
        .sync_events(events_first, &sink, None)
        .expect("second sync");

    assert_eq!(first.saved, 2);
    assert_eq!(second.deduplicated, 2);
    assert_eq!(sink.events_len(), 2);
}

#[test]
fn fetch_artifact_populates_hash_length_and_mime() {
    let provider = EdinetProvider::new(
        ucel_ir::http::HttpClient::new(test_http_config()).expect("http client"),
        EdinetConfig {
            api_key: Some("test-key".to_string()),
            fixtures_dir: Some(fixture_dir()),
            list_url: "http://unused".to_string(),
        },
    );
    let sink = MemorySink::default();
    let artifact = provider
        .fetch_artifact(
            &FetchArtifactRequest {
                date: "2026-01-01".to_string(),
                doc_id: "S100AAA1".to_string(),
                key: "edinet/2026-01-01/S100AAA1".to_string(),
            },
            &sink,
        )
        .expect("artifact should be written");

    assert_eq!(artifact.content_length, Some(29));
    assert_eq!(artifact.mime.as_deref(), Some("application/zip"));
    assert!(artifact.sha256.as_ref().is_some_and(|v| !v.is_empty()));
    assert!(artifact.retrieved_at.is_some());
}
