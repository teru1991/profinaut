pub mod checkpoint;
pub mod market_meta_store;
pub mod replay;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;
use ucel_symbol_core::{
    cmp_decimal, InstrumentId, MarketMeta, Snapshot, StandardizedInstrument, SymbolStatus,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub store_version: u64,
    pub ts_recv: SystemTime,
    pub instruments: Vec<StandardizedInstrument>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolEvent {
    Added {
        instrument: StandardizedInstrument,
        ts_recv: SystemTime,
        ts_event: Option<SystemTime>,
        schema_version: u16,
        store_version: u64,
    },
    Removed {
        id: InstrumentId,
        last_known: Option<StandardizedInstrument>,
        reason: Option<String>,
        ts_recv: SystemTime,
        store_version: u64,
    },
    StatusChanged {
        id: InstrumentId,
        from: SymbolStatus,
        to: SymbolStatus,
        ts_recv: SystemTime,
        store_version: u64,
    },
    ParamChanged {
        id: InstrumentId,
        changed_fields: Vec<String>,
        before: StandardizedInstrument,
        after: Box<StandardizedInstrument>,
        ts_recv: SystemTime,
        store_version: u64,
    },
}

pub struct SymbolStore {
    instruments: DashMap<InstrumentId, StandardizedInstrument>,
    market_meta: DashMap<InstrumentId, MarketMeta>,
    store_version: AtomicU64,
    event_log: Mutex<Vec<replay::VersionedSymbolEvent>>,
}

impl Default for SymbolStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolStore {
    pub fn new() -> Self {
        Self {
            instruments: DashMap::new(),
            market_meta: DashMap::new(),
            store_version: AtomicU64::new(0),
            event_log: Mutex::new(Vec::new()),
        }
    }

    pub fn version(&self) -> u64 {
        self.store_version.load(Ordering::SeqCst)
    }

    pub fn store_version(&self) -> checkpoint::StoreVersion {
        self.version()
    }

    pub fn get_market_meta(&self, id: &InstrumentId) -> Option<MarketMeta> {
        self.market_meta.get(id).map(|v| v.clone())
    }

    pub fn list_market_meta_ids(&self) -> Vec<InstrumentId> {
        self.market_meta.iter().map(|e| e.key().clone()).collect()
    }

    pub fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            store_version: self.version(),
            ts_recv: SystemTime::now(),
            instruments: self
                .instruments
                .iter()
                .map(|entry| entry.value().clone())
                .collect(),
        }
    }

    pub fn apply_snapshot(&self, snapshot: Snapshot) -> Vec<SymbolEvent> {
        self.apply_snapshot_with_meta_whitelist(snapshot, &[])
    }

    pub fn apply_snapshot_with_meta_whitelist(
        &self,
        snapshot: Snapshot,
        meta_whitelist: &[&str],
    ) -> Vec<SymbolEvent> {
        let mut events = Vec::new();
        let mut incoming = BTreeMap::new();
        for instrument in snapshot.instruments {
            incoming.insert(instrument.id.clone(), instrument);
        }

        let stale_ids: Vec<InstrumentId> = self
            .instruments
            .iter()
            .filter_map(|current| {
                if incoming.contains_key(current.key()) {
                    None
                } else {
                    Some(current.key().clone())
                }
            })
            .collect();

        for stale_id in stale_ids {
            if let Some((_, prev)) = self.instruments.remove(&stale_id) {
                self.market_meta.remove(&stale_id);
                let version = self.bump_version();
                let event = SymbolEvent::Removed {
                    id: prev.id.clone(),
                    last_known: Some(prev),
                    reason: Some("snapshot_missing".into()),
                    ts_recv: SystemTime::now(),
                    store_version: version,
                };
                self.record_event(&event);
                events.push(event);
            }
        }

        for (id, instrument) in incoming {
            if let Some(existing) = self.instruments.get(&id) {
                let before = existing.clone();
                drop(existing);
                if before.status != instrument.status {
                    self.instruments.insert(id.clone(), instrument.clone());
                    self.market_meta
                        .insert(id.clone(), MarketMeta::from(&instrument));
                    let version = self.bump_version();
                    let event = SymbolEvent::StatusChanged {
                        id,
                        from: before.status,
                        to: instrument.status,
                        ts_recv: SystemTime::now(),
                        store_version: version,
                    };
                    self.record_event(&event);
                    events.push(event);
                    continue;
                }

                let changed_fields = changed_param_fields(&before, &instrument, meta_whitelist);
                if !changed_fields.is_empty() {
                    self.instruments.insert(id.clone(), instrument.clone());
                    self.market_meta
                        .insert(id.clone(), MarketMeta::from(&instrument));
                    let version = self.bump_version();
                    let event = SymbolEvent::ParamChanged {
                        id,
                        changed_fields,
                        before,
                        after: Box::new(instrument),
                        ts_recv: SystemTime::now(),
                        store_version: version,
                    };
                    self.record_event(&event);
                    events.push(event);
                }
            } else {
                self.instruments.insert(id.clone(), instrument.clone());
                self.market_meta.insert(id, MarketMeta::from(&instrument));
                let version = self.bump_version();
                let event = SymbolEvent::Added {
                    instrument: instrument.clone(),
                    ts_recv: instrument.ts_recv,
                    ts_event: instrument.ts_event,
                    schema_version: instrument.schema_version,
                    store_version: version,
                };
                self.record_event(&event);
                events.push(event);
            }
        }

        events
    }

    fn bump_version(&self) -> u64 {
        self.store_version.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn record_event(&self, event: &SymbolEvent) {
        let mut event_log = self.event_log.lock().expect("event_log lock");
        event_log.push(replay::VersionedSymbolEvent {
            store_version: event_store_version(event),
            event: event.clone(),
        });
    }

    pub fn checkpoint(&self) -> checkpoint::StoreCheckpoint {
        let mut rs = replay::ReplayState::new(checkpoint::SchemaVersion(1));
        let event_log = self.event_log.lock().expect("event_log lock");
        for ev in event_log.iter() {
            rs.apply(ev).expect("internal event log must be contiguous");
        }
        rs.checkpoint()
    }

    pub fn export_since(
        &self,
        from_version: checkpoint::StoreVersion,
    ) -> Vec<replay::VersionedSymbolEvent> {
        let event_log = self.event_log.lock().expect("event_log lock");
        event_log
            .iter()
            .filter(|e| e.store_version > from_version)
            .cloned()
            .collect()
    }

    pub fn import_events(
        &self,
        schema_version: checkpoint::SchemaVersion,
        events: &[replay::VersionedSymbolEvent],
    ) -> Result<checkpoint::StoreCheckpoint, checkpoint::CheckpointError> {
        let mut rs = replay::ReplayState::new(schema_version);
        for ev in events {
            rs.apply(ev)?;
            self.apply_symbol_event(&ev.event);
            self.store_version.store(ev.store_version, Ordering::SeqCst);
        }

        {
            let mut event_log = self.event_log.lock().expect("event_log lock");
            *event_log = events.to_vec();
        }

        Ok(rs.checkpoint())
    }

    fn apply_symbol_event(&self, event: &SymbolEvent) {
        match event {
            SymbolEvent::Added { instrument, .. } => {
                self.instruments
                    .insert(instrument.id.clone(), instrument.clone());
                self.market_meta
                    .insert(instrument.id.clone(), MarketMeta::from(instrument));
            }
            SymbolEvent::Removed { id, .. } => {
                self.instruments.remove(id);
                self.market_meta.remove(id);
            }
            SymbolEvent::StatusChanged { id, to, .. } => {
                if let Some(mut existing) = self.instruments.get_mut(id) {
                    existing.status = to.clone();
                    let updated = existing.clone();
                    drop(existing);
                    self.market_meta
                        .insert(id.clone(), MarketMeta::from(&updated));
                }
            }
            SymbolEvent::ParamChanged { id, after, .. } => {
                self.instruments.insert(id.clone(), (**after).clone());
                self.market_meta
                    .insert(id.clone(), MarketMeta::from(&**after));
            }
        }
    }
}

fn event_store_version(event: &SymbolEvent) -> u64 {
    match event {
        SymbolEvent::Added { store_version, .. }
        | SymbolEvent::Removed { store_version, .. }
        | SymbolEvent::StatusChanged { store_version, .. }
        | SymbolEvent::ParamChanged { store_version, .. } => *store_version,
    }
}

fn changed_param_fields(
    before: &StandardizedInstrument,
    after: &StandardizedInstrument,
    meta_whitelist: &[&str],
) -> Vec<String> {
    let mut fields = BTreeSet::new();
    if cmp_decimal(before.tick_size, after.tick_size).is_ne() {
        fields.insert("tick_size".to_string());
    }
    if cmp_decimal(before.lot_size, after.lot_size).is_ne() {
        fields.insert("lot_size".to_string());
    }
    if before
        .min_order_qty
        .zip(after.min_order_qty)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.min_order_qty != after.min_order_qty)
    {
        fields.insert("min_order_qty".to_string());
    }
    if before
        .max_order_qty
        .zip(after.max_order_qty)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.max_order_qty != after.max_order_qty)
    {
        fields.insert("max_order_qty".to_string());
    }
    if before
        .min_notional
        .zip(after.min_notional)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.min_notional != after.min_notional)
    {
        fields.insert("min_notional".to_string());
    }
    if before
        .contract_size
        .zip(after.contract_size)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.contract_size != after.contract_size)
    {
        fields.insert("contract_size".to_string());
    }
    if before.price_precision != after.price_precision {
        fields.insert("price_precision".to_string());
    }
    if before.qty_precision != after.qty_precision {
        fields.insert("qty_precision".to_string());
    }
    for key in meta_whitelist {
        if before.meta.get(*key) != after.meta.get(*key) {
            fields.insert(format!("meta.{key}"));
        }
    }
    fields.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use ucel_symbol_core::{Exchange, InstrumentMeta, MarketType, SnapshotOrigin, SnapshotSource};

    fn instrument(
        symbol: &str,
        status: SymbolStatus,
        market_type: MarketType,
    ) -> StandardizedInstrument {
        let id = InstrumentId {
            exchange: Exchange::Binance,
            market_type: market_type.clone(),
            raw_symbol: symbol.to_string(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        };
        StandardizedInstrument {
            id,
            exchange: Exchange::Binance,
            market_type,
            base: "BTC".into(),
            quote: "USDT".into(),
            raw_symbol: symbol.into(),
            status,
            tick_size: Decimal::new(1, 2),
            lot_size: Decimal::new(1, 3),
            min_order_qty: None,
            max_order_qty: None,
            min_notional: None,
            price_precision: Some(2),
            qty_precision: Some(3),
            contract_size: None,
            meta: InstrumentMeta::new(),
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: 1,
        }
    }

    fn snapshot(instruments: Vec<StandardizedInstrument>) -> Snapshot {
        Snapshot {
            snapshot_id: "s1".into(),
            ts_recv: SystemTime::now(),
            instruments,
            origin: SnapshotOrigin {
                source: SnapshotSource::Rest,
                restored: false,
            },
        }
    }

    #[test]
    fn diff_and_version_monotonic() {
        let store = SymbolStore::new();
        let first = instrument("BTCUSDT", SymbolStatus::Trading, MarketType::Spot);
        let events = store.apply_snapshot(snapshot(vec![first.clone()]));
        assert!(matches!(events[0], SymbolEvent::Added { .. }));
        assert_eq!(store.version(), 1);

        let mut changed = first.clone();
        changed.status = SymbolStatus::Suspended;
        let events = store.apply_snapshot(snapshot(vec![changed.clone()]));
        assert!(matches!(events[0], SymbolEvent::StatusChanged { .. }));
        assert_eq!(store.version(), 2);

        let mut param = changed.clone();
        param.tick_size = Decimal::new(5, 2);
        let events = store.apply_snapshot(snapshot(vec![param.clone()]));
        assert!(matches!(events[0], SymbolEvent::ParamChanged { .. }));
        assert_eq!(store.version(), 3);

        let events = store.apply_snapshot(snapshot(vec![]));
        assert!(matches!(events[0], SymbolEvent::Removed { .. }));
        assert_eq!(store.version(), 4);
    }

    #[test]
    fn instrument_id_distinguishes_market_type() {
        let store = SymbolStore::new();
        let spot = instrument("BTCUSDT", SymbolStatus::Trading, MarketType::Spot);
        let perp = instrument(
            "BTCUSDT",
            SymbolStatus::Trading,
            MarketType::LinearPerpetual,
        );
        let events = store.apply_snapshot(snapshot(vec![spot, perp]));
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn meta_is_ignored_unless_whitelisted() {
        let store = SymbolStore::new();
        let first = instrument("BTCUSDT", SymbolStatus::Trading, MarketType::Spot);
        store.apply_snapshot(snapshot(vec![first.clone()]));

        let mut changed = first.clone();
        changed
            .meta
            .insert("important".into(), serde_json::json!("changed"));

        let events = store.apply_snapshot(snapshot(vec![changed.clone()]));
        assert!(events.is_empty());

        let events =
            store.apply_snapshot_with_meta_whitelist(snapshot(vec![changed]), &["important"]);
        assert!(matches!(events[0], SymbolEvent::ParamChanged { .. }));
    }
}

pub use checkpoint::{CheckpointError, SchemaVersion, StoreCheckpoint, StoreVersion};
pub use market_meta_store::{MarketMetaEvent, MarketMetaRegistrySnapshot, MarketMetaStore};
pub use replay::{ReplayState, VersionedSymbolEvent};

#[cfg(test)]
mod market_meta_store_tests {
    use super::*;
    use rust_decimal::Decimal;
    use ucel_symbol_core::{Exchange, InstrumentMeta, MarketType, SnapshotOrigin, SnapshotSource};

    fn mk_inst(
        symbol: &str,
        tick: &str,
        step: &str,
        min_qty: Option<&str>,
    ) -> StandardizedInstrument {
        StandardizedInstrument {
            id: InstrumentId {
                exchange: Exchange::Gmocoin,
                market_type: MarketType::Spot,
                raw_symbol: symbol.to_string(),
                expiry: None,
                strike: None,
                option_right: None,
                contract_size: None,
            },
            exchange: Exchange::Gmocoin,
            market_type: MarketType::Spot,
            base: symbol.to_string(),
            quote: "JPY".into(),
            raw_symbol: symbol.to_string(),
            status: SymbolStatus::Trading,
            tick_size: tick.parse::<Decimal>().unwrap(),
            lot_size: step.parse::<Decimal>().unwrap(),
            min_order_qty: min_qty.map(|v| v.parse::<Decimal>().unwrap()),
            max_order_qty: None,
            min_notional: None,
            price_precision: None,
            qty_precision: None,
            contract_size: None,
            meta: InstrumentMeta::new(),
            ts_recv: SystemTime::now(),
            ts_event: None,
            schema_version: ucel_symbol_core::SYMBOL_SCHEMA_VERSION,
        }
    }

    #[test]
    fn market_meta_is_synced_on_apply_snapshot() {
        let store = SymbolStore::new();
        let s1 = Snapshot {
            snapshot_id: "s1".into(),
            ts_recv: SystemTime::now(),
            instruments: vec![mk_inst("BTC", "0.1", "0.001", Some("0.01"))],
            origin: SnapshotOrigin {
                source: SnapshotSource::Rest,
                restored: false,
            },
        };
        store.apply_snapshot(s1);
        let id = InstrumentId {
            exchange: Exchange::Gmocoin,
            market_type: MarketType::Spot,
            raw_symbol: "BTC".into(),
            expiry: None,
            strike: None,
            option_right: None,
            contract_size: None,
        };
        let mm = store.get_market_meta(&id).expect("market meta must exist");
        assert_eq!(mm.tick_size.to_string(), "0.1");
        assert_eq!(mm.step_size.to_string(), "0.001");
        assert_eq!(mm.min_qty.unwrap().to_string(), "0.01");

        let s2 = Snapshot {
            snapshot_id: "s2".into(),
            ts_recv: SystemTime::now(),
            instruments: vec![mk_inst("BTC", "0.01", "0.0001", Some("0.001"))],
            origin: SnapshotOrigin {
                source: SnapshotSource::Rest,
                restored: false,
            },
        };
        let ev = store.apply_snapshot(s2);
        assert!(!ev.is_empty());

        let mm2 = store.get_market_meta(&id).expect("market meta must exist");
        assert_eq!(mm2.tick_size.to_string(), "0.01");
        assert_eq!(mm2.step_size.to_string(), "0.0001");
        assert_eq!(mm2.min_qty.unwrap().to_string(), "0.001");

        let s3 = Snapshot {
            snapshot_id: "s3".into(),
            ts_recv: SystemTime::now(),
            instruments: vec![],
            origin: SnapshotOrigin {
                source: SnapshotSource::Rest,
                restored: false,
            },
        };
        store.apply_snapshot(s3);
        assert!(store.get_market_meta(&id).is_none());
    }
}
