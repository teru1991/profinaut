use ucel_ir::{
    CanonicalEntityId, HttpConfig, IrEvent, IrProvider, MemorySink, Quality, UcelIrClient,
    UcelIrConfig,
};

fn sample_event() -> IrEvent {
    IrEvent {
        provider: IrProvider::Edinet,
        source_event_id: "doc-001".to_string(),
        entity_id: CanonicalEntityId::EdinetCode("E00001".to_string()),
        entity_aliases: vec![],
        filing_type: "annual".to_string(),
        filing_date: Some("2026-01-01".to_string()),
        published_at: Some(1_738_000_000),
        observed_at: 1_738_000_100,
        artifacts: vec![],
        quality: Quality::default(),
        trace_id: "trace-smoke".to_string(),
    }
}

#[test]
fn sync_once_is_idempotent_for_duplicate_events() {
    let client = UcelIrClient::new(UcelIrConfig {
        http: HttpConfig {
            user_agent: "ucel-ir-test/0.1".to_string(),
            timeout_ms: 1_000,
            max_retries: 1,
            base_backoff_ms: 10,
            rate_limit_per_sec: 10,
        },
        raw_storage_root: "/tmp/ucel-ir/raw".to_string(),
        checkpoint_path: "/tmp/ucel-ir/checkpoint".to_string(),
    })
    .expect("client must initialize");

    let sink = MemorySink::default();

    let first = client
        .sync_events(vec![sample_event()], &sink, None)
        .expect("first sync should succeed");
    let second = client
        .sync_events(vec![sample_event()], &sink, None)
        .expect("second sync should succeed");

    assert_eq!(first.saved, 1);
    assert_eq!(first.deduplicated, 0);
    assert_eq!(second.saved, 0);
    assert_eq!(second.deduplicated, 1);
    assert_eq!(sink.events_len(), 1);
}
