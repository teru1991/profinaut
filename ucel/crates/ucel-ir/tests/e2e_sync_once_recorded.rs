use std::path::PathBuf;

use ucel_ir::{
    EdinetConfig, EdinetSyncConfig, HttpConfig, MemoryCheckpointStore, MemorySink, SecEdgarConfig,
    SecEdgarSyncConfig, SyncRequest, UcelIrClient, UcelIrConfig,
};

fn base_client() -> UcelIrClient {
    UcelIrClient::new(UcelIrConfig {
        http: HttpConfig {
            user_agent: "ucel-ir-e2e-tests/0.1".to_string(),
            timeout_ms: 1_000,
            max_retries: 1,
            base_backoff_ms: 10,
            rate_limit_per_sec: 10,
        },
        raw_storage_root: "/tmp/ucel-ir/raw".to_string(),
        checkpoint_path: "/tmp/ucel-ir/checkpoint".to_string(),
    })
    .expect("client")
}

fn fixtures(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn sync_once_runs_edinet_and_sec_together() {
    let client = base_client();
    let request = SyncRequest {
        edinet: Some(EdinetSyncConfig {
            date: "2026-01-01".to_string(),
            config: EdinetConfig {
                api_key: None,
                fixtures_dir: Some(fixtures("testdata/edinet")),
                list_url: "http://unused".to_string(),
            },
        }),
        sec_edgar: Some(SecEdgarSyncConfig {
            config: {
                let mut cfg = SecEdgarConfig::with_defaults(
                    "ucel-ir-e2e-tests/0.1",
                    fixtures("testdata/sec"),
                );
                cfg.tickers = vec!["AAPL".to_string()];
                cfg
            },
        }),
        fetch_artifacts: true,
    };

    let sink = MemorySink::default();
    let checkpoints = MemoryCheckpointStore::default();
    let report = client
        .sync_once(&request, &sink, &sink, &checkpoints)
        .expect("sync once");

    assert_eq!(report.providers["edinet"].detected, 2);
    assert_eq!(report.providers["sec_edgar"].detected, 2);
    assert_eq!(sink.events_len(), 4);
}

#[test]
fn bulkhead_edinet_failure_does_not_block_sec() {
    let client = base_client();
    let request = SyncRequest {
        edinet: Some(EdinetSyncConfig {
            date: "2026-01-02".to_string(),
            config: EdinetConfig {
                api_key: None,
                fixtures_dir: Some(fixtures("testdata/edinet")),
                list_url: "http://unused".to_string(),
            },
        }),
        sec_edgar: Some(SecEdgarSyncConfig {
            config: {
                let mut cfg = SecEdgarConfig::with_defaults(
                    "ucel-ir-e2e-tests/0.1",
                    fixtures("testdata/sec"),
                );
                cfg.tickers = vec!["AAPL".to_string()];
                cfg
            },
        }),
        fetch_artifacts: false,
    };

    let sink = MemorySink::default();
    let checkpoints = MemoryCheckpointStore::default();
    let report = client
        .sync_once(&request, &sink, &sink, &checkpoints)
        .expect("sync once");

    assert!(report.providers["edinet"].degraded > 0);
    assert_eq!(report.providers["sec_edgar"].saved, 2);
}

#[test]
fn bulkhead_sec_failure_does_not_block_edinet() {
    let client = base_client();
    let request = SyncRequest {
        edinet: Some(EdinetSyncConfig {
            date: "2026-01-01".to_string(),
            config: EdinetConfig {
                api_key: None,
                fixtures_dir: Some(fixtures("testdata/edinet")),
                list_url: "http://unused".to_string(),
            },
        }),
        sec_edgar: Some(SecEdgarSyncConfig {
            config: {
                let mut cfg = SecEdgarConfig::with_defaults(
                    "ucel-ir-e2e-tests/0.1",
                    fixtures("testdata/sec_bad"),
                );
                cfg.tickers = vec!["AAPL".to_string()];
                cfg
            },
        }),
        fetch_artifacts: false,
    };

    let sink = MemorySink::default();
    let checkpoints = MemoryCheckpointStore::default();
    let report = client
        .sync_once(&request, &sink, &sink, &checkpoints)
        .expect("sync once");

    assert_eq!(report.providers["edinet"].saved, 2);
    assert!(report.providers["sec_edgar"].degraded > 0);
}
