use std::collections::BTreeMap;
use ucel_core::{
    apply_orderbook_delta, guard_orderbook, CanonicalOrderBookDelta, CanonicalOrderBookSnapshot,
    PublicAdapterSupport,
};

pub fn public_adapter_support_matrix() -> BTreeMap<&'static str, PublicAdapterSupport> {
    BTreeMap::from([
        ("binance", PublicAdapterSupport::Supported),
        ("binance-usdm", PublicAdapterSupport::Supported),
        ("binance-coinm", PublicAdapterSupport::Supported),
        ("binance-options", PublicAdapterSupport::Partial),
        ("bithumb", PublicAdapterSupport::Partial),
        ("bitmex", PublicAdapterSupport::Supported),
        ("deribit", PublicAdapterSupport::Supported),
        ("coinbase", PublicAdapterSupport::Supported),
        ("bybit", PublicAdapterSupport::Supported),
        ("okx", PublicAdapterSupport::Supported),
        ("htx", PublicAdapterSupport::Supported),
        ("kraken", PublicAdapterSupport::Supported),
        ("bitbank", PublicAdapterSupport::Partial),
        ("bitflyer", PublicAdapterSupport::Partial),
        ("coincheck", PublicAdapterSupport::Partial),
        ("gmocoin", PublicAdapterSupport::Partial),
        ("bittrade", PublicAdapterSupport::Partial),
        ("sbivc", PublicAdapterSupport::Partial),
        ("upbit", PublicAdapterSupport::Supported),
    ])
}

pub fn assert_apply_and_guard(
    snapshot: &CanonicalOrderBookSnapshot,
    delta: &CanonicalOrderBookDelta,
) -> CanonicalOrderBookSnapshot {
    let next = apply_orderbook_delta(snapshot, delta);
    guard_orderbook(&next).expect("orderbook guard");
    next
}
