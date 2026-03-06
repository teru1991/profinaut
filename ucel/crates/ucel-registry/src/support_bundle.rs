use crate::hub::{ExchangeId, Hub};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub fn hub_bundle(hub: &Hub) -> serde_json::Value {
    let mut venues = Vec::new();
    for exchange in [
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::Coinbase,
        ExchangeId::Coincheck,
        ExchangeId::Deribit,
        ExchangeId::Gmocoin,
        ExchangeId::Kraken,
        ExchangeId::Okx,
        ExchangeId::Upbit,
    ] {
        let ops = hub.list_operations(exchange).unwrap_or_default();
        let channels = hub.list_channels(exchange).unwrap_or_default();
        venues.push(serde_json::json!({
            "exchange_id": exchange.as_str(),
            "operations": ops.len(),
            "channels": channels.len(),
        }));
    }

    let repo_root = detect_repo_root();
    let mut hashes = ucel_diagnostics_core::default_hash_set(&repo_root)
        .unwrap_or_else(|_| ucel_core::BundleHashSet {
            coverage_hash: hash_literal("coverage-fallback"),
            coverage_v2_hash: hash_literal("coverage-v2-fallback"),
            ws_rules_hash: hash_literal("ws-rules-fallback"),
            catalog_hash: hash_literal("catalog-fallback"),
            policy_hash: hash_literal("policy-fallback"),
            symbol_meta_hash: hash_literal("symbol-meta-fallback"),
            execution_surface_hash: hash_literal("execution-fallback"),
            runtime_capability_hash: hash_literal("runtime-fallback"),
        });

    let runtime_capability_hash = hash_literal(&serde_json::to_string(&venues).unwrap_or_default());
    hashes.runtime_capability_hash = runtime_capability_hash.clone();

    serde_json::json!({
        "diag_semver": ucel_diagnostics_core::DIAG_SEMVER_STR,
        "ssot": {
            "rules_version": "v1",
            "coverage_hash": hashes.coverage_hash,
            "coverage_v2_hash": hashes.coverage_v2_hash,
            "ws_rules_hash": hashes.ws_rules_hash,
            "catalog_hash": hashes.catalog_hash,
            "policy_hash": hashes.policy_hash,
            "symbol_meta_hash": hashes.symbol_meta_hash,
            "execution_surface_hash": hashes.execution_surface_hash,
            "runtime_capability_hash": hashes.runtime_capability_hash,
        },
        "venues": venues
    })
}

fn hash_literal(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

fn detect_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}
