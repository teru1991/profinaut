use ucel_symbol_core::{Exchange, InstrumentId, MarketType};
use ucel_symbol_store::{
    ReplayState, SchemaVersion, StoreVersion, SymbolEvent, VersionedSymbolEvent,
};

#[test]
fn replay_detects_gap_and_out_of_order() {
    let mut rs = ReplayState::new(SchemaVersion(1));

    let e1 = VersionedSymbolEvent {
        store_version: 1,
        event: dummy_event("A", 1),
    };
    let e3 = VersionedSymbolEvent {
        store_version: 3,
        event: dummy_event("B", 3),
    };
    let e2 = VersionedSymbolEvent {
        store_version: 2,
        event: dummy_event("C", 2),
    };

    rs.apply(&e1).unwrap();

    let err = rs.apply(&e3).unwrap_err();
    assert!(format!("{err:?}").contains("ReplayGap"));

    let mut rs2 = ReplayState::new(SchemaVersion(1));
    rs2.apply(&e1).unwrap();
    let e0 = VersionedSymbolEvent {
        store_version: 1,
        event: dummy_event("X", 1),
    };
    let err2 = rs2.apply(&e0).unwrap_err();
    assert!(format!("{err2:?}").contains("ReplayOutOfOrder"));

    let mut rs3 = ReplayState::new(SchemaVersion(1));
    rs3.apply(&e1).unwrap();
    rs3.apply(&e2).unwrap();
}

#[test]
fn checkpoint_is_stable_for_same_event_sequence() {
    let mut rs1 = ReplayState::new(SchemaVersion(1));
    let mut rs2 = ReplayState::new(SchemaVersion(1));

    let seq = vec![
        VersionedSymbolEvent {
            store_version: 1,
            event: dummy_event("A", 1),
        },
        VersionedSymbolEvent {
            store_version: 2,
            event: dummy_event("B", 2),
        },
        VersionedSymbolEvent {
            store_version: 3,
            event: dummy_event("C", 3),
        },
    ];

    for ev in &seq {
        rs1.apply(ev).unwrap();
        rs2.apply(ev).unwrap();
    }

    let c1 = rs1.checkpoint();
    let c2 = rs2.checkpoint();
    assert_eq!(c1.schema_version.0, 1);
    assert_eq!(c1.store_version, 3 as StoreVersion);
    assert_eq!(c1, c2);
}

fn dummy_event(tag: &str, store_version: u64) -> SymbolEvent {
    SymbolEvent::Removed {
        id: InstrumentId {
            exchange: Exchange::Binance,
            market_type: MarketType::Spot,
            raw_symbol: format!("SYM-{tag}"),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        },
        last_known: None,
        reason: Some(format!("reason-{tag}")),
        ts_recv: std::time::SystemTime::UNIX_EPOCH,
        store_version,
    }
}
