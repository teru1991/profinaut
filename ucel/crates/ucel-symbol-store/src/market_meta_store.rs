use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};
use ucel_symbol_core::{cmp_decimal, MarketMeta, MarketMetaId, MarketMetaSnapshot};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMetaRegistrySnapshot {
    pub store_version: u64,
    pub ts_recv: SystemTime,
    pub markets: Vec<MarketMeta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketMetaEvent {
    Added {
        id: MarketMetaId,
        meta: MarketMeta,
        ts_recv: SystemTime,
        store_version: u64,
    },
    Updated {
        id: MarketMetaId,
        changed_fields: Vec<String>,
        before: MarketMeta,
        after: MarketMeta,
        ts_recv: SystemTime,
        store_version: u64,
    },
    Removed {
        id: MarketMetaId,
        last_known: Option<MarketMeta>,
        reason: &'static str,
        ts_recv: SystemTime,
        store_version: u64,
    },
    Expired {
        id: MarketMetaId,
        last_known: Option<MarketMeta>,
        ts_recv: SystemTime,
        store_version: u64,
    },
}

#[derive(Clone)]
struct Entry {
    meta: MarketMeta,
    expires_at: Instant,
}

pub struct MarketMetaStore {
    map: DashMap<MarketMetaId, Entry>,
    store_version: AtomicU64,
    ttl: Duration,
}

impl MarketMetaStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: DashMap::new(),
            store_version: AtomicU64::new(0),
            ttl,
        }
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    pub fn version(&self) -> u64 {
        self.store_version.load(Ordering::SeqCst)
    }

    pub fn snapshot(&self) -> MarketMetaRegistrySnapshot {
        MarketMetaRegistrySnapshot {
            store_version: self.version(),
            ts_recv: SystemTime::now(),
            markets: self.map.iter().map(|e| e.value().meta.clone()).collect(),
        }
    }

    pub fn gc_expired(&self) -> Vec<MarketMetaEvent> {
        let mut events = Vec::new();
        let now = Instant::now();
        let keys: Vec<MarketMetaId> = self
            .map
            .iter()
            .filter_map(|e| {
                if e.value().expires_at <= now {
                    Some(e.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for k in keys {
            if let Some((_, entry)) = self.map.remove(&k) {
                let ver = self.bump_version();
                events.push(MarketMetaEvent::Expired {
                    id: k,
                    last_known: Some(entry.meta),
                    ts_recv: SystemTime::now(),
                    store_version: ver,
                });
            }
        }
        events
    }

    pub fn apply_snapshot_full(&self, snapshot: MarketMetaSnapshot) -> Vec<MarketMetaEvent> {
        self.apply_snapshot(snapshot, true)
    }

    pub fn apply_snapshot_partial(&self, snapshot: MarketMetaSnapshot) -> Vec<MarketMetaEvent> {
        self.apply_snapshot(snapshot, false)
    }

    fn apply_snapshot(&self, snapshot: MarketMetaSnapshot, is_full: bool) -> Vec<MarketMetaEvent> {
        let mut events = Vec::new();

        // 入力を map 化（同一キーが来たら後勝ち）
        let mut incoming: BTreeMap<MarketMetaId, MarketMeta> = BTreeMap::new();
        for m in snapshot.markets {
            // 安全優先: validate_meta() 失敗は受入拒否 (skip)
            if m.validate_meta().is_ok() {
                incoming.insert(m.id.clone(), m);
            }
        }

        if is_full {
            // snapshotに無いものは削除
            let stale: Vec<MarketMetaId> = self
                .map
                .iter()
                .filter_map(|cur| {
                    if incoming.contains_key(cur.key()) {
                        None
                    } else {
                        Some(cur.key().clone())
                    }
                })
                .collect();

            for k in stale {
                if let Some((_, entry)) = self.map.remove(&k) {
                    let ver = self.bump_version();
                    events.push(MarketMetaEvent::Removed {
                        id: k,
                        last_known: Some(entry.meta),
                        reason: "snapshot_missing",
                        ts_recv: SystemTime::now(),
                        store_version: ver,
                    });
                }
            }
        }

        // upsert
        for (id, meta) in incoming {
            let expires_at = Instant::now() + self.ttl;

            if let Some(existing) = self.map.get(&id) {
                let before = existing.meta.clone();
                drop(existing);

                let changed_fields = changed_meta_fields(&before, &meta);
                if !changed_fields.is_empty() {
                    self.map.insert(
                        id.clone(),
                        Entry {
                            meta: meta.clone(),
                            expires_at,
                        },
                    );
                    let ver = self.bump_version();
                    events.push(MarketMetaEvent::Updated {
                        id,
                        changed_fields,
                        before,
                        after: meta,
                        ts_recv: SystemTime::now(),
                        store_version: ver,
                    });
                } else {
                    // 変化なしでも TTL 延長
                    self.map.insert(
                        id,
                        Entry {
                            meta: before,
                            expires_at,
                        },
                    );
                }
            } else {
                self.map.insert(
                    id.clone(),
                    Entry {
                        meta: meta.clone(),
                        expires_at,
                    },
                );
                let ver = self.bump_version();
                events.push(MarketMetaEvent::Added {
                    id,
                    meta,
                    ts_recv: SystemTime::now(),
                    store_version: ver,
                });
            }
        }

        events
    }

    /// 期限切れなら None。期限内なら clone を返す
    pub fn get(&self, id: &MarketMetaId) -> Option<MarketMeta> {
        let now = Instant::now();
        if let Some(entry) = self.map.get(id) {
            if entry.expires_at <= now {
                drop(entry);
                let _ = self.map.remove(id);
                return None;
            }
            return Some(entry.meta.clone());
        }
        None
    }

    pub fn get_by_parts(
        &self,
        exchange: ucel_symbol_core::Exchange,
        market_type: ucel_symbol_core::MarketType,
        raw_symbol: &str,
    ) -> Option<MarketMeta> {
        let id = MarketMetaId::new(exchange, market_type, raw_symbol.to_string());
        self.get(&id)
    }

    fn bump_version(&self) -> u64 {
        self.store_version.fetch_add(1, Ordering::SeqCst) + 1
    }
}

fn changed_meta_fields(before: &MarketMeta, after: &MarketMeta) -> Vec<String> {
    let mut fields = BTreeSet::new();

    if before.base != after.base {
        fields.insert("base".to_string());
    }
    if before.quote != after.quote {
        fields.insert("quote".to_string());
    }

    if cmp_decimal(before.tick_size, after.tick_size).is_ne() {
        fields.insert("tick_size".to_string());
    }
    if cmp_decimal(before.step_size, after.step_size).is_ne() {
        fields.insert("step_size".to_string());
    }

    if before
        .min_qty
        .zip(after.min_qty)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.min_qty != after.min_qty)
    {
        fields.insert("min_qty".to_string());
    }
    if before
        .max_qty
        .zip(after.max_qty)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.max_qty != after.max_qty)
    {
        fields.insert("max_qty".to_string());
    }
    if before
        .min_notional
        .zip(after.min_notional)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.min_notional != after.min_notional)
    {
        fields.insert("min_notional".to_string());
    }

    if before.price_precision != after.price_precision {
        fields.insert("price_precision".to_string());
    }
    if before.qty_precision != after.qty_precision {
        fields.insert("qty_precision".to_string());
    }

    if before
        .contract_size
        .zip(after.contract_size)
        .map(|(a, b)| cmp_decimal(a, b).is_ne())
        .unwrap_or(before.contract_size != after.contract_size)
    {
        fields.insert("contract_size".to_string());
    }

    if before.meta != after.meta {
        fields.insert("meta".to_string());
    }

    fields.into_iter().collect()
}
