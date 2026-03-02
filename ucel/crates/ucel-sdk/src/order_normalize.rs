use thiserror::Error;
use ucel_core::order_gate::{OrderGate, OrderGateError};
use ucel_core::{Decimal, Side, StepSize, TickSize};
use ucel_symbol_core::{MarketMeta, MarketMetaId, OrderSide as MetaSide};
use ucel_symbol_store::MarketMetaStore;

#[derive(Debug, Error)]
pub enum OrderNormalizeError {
    #[error("market meta not found: {0:?}")]
    MetaNotFound(MarketMetaId),
    #[error("market meta invalid: {0}")]
    MetaInvalid(String),
    #[error("order gate error: {0}")]
    Gate(#[from] OrderGateError),
}

fn to_side(s: MetaSide) -> Side {
    match s {
        MetaSide::Buy => Side::Buy,
        MetaSide::Sell => Side::Sell,
    }
}

pub fn normalize_limit_with_meta(
    gate: &OrderGate,
    meta: &MarketMeta,
    side: MetaSide,
    price: Decimal,
    qty: Decimal,
) -> Result<(Decimal, Decimal), OrderNormalizeError> {
    meta.validate_meta()
        .map_err(|e| OrderNormalizeError::MetaInvalid(e.to_string()))?;

    let tick = TickSize(meta.tick_size);
    let step = StepSize(meta.step_size);

    let core_side = to_side(side);
    let (price_mode, qty_mode) = OrderGate::recommended_modes(core_side.clone());
    let (p, q) = gate.quantize_limit(core_side, price, qty, tick, step, price_mode, qty_mode)?;

    meta.validate_order(p.as_decimal(), q.as_decimal())
        .map_err(|e| OrderNormalizeError::MetaInvalid(e.to_string()))?;

    Ok((p.as_decimal(), q.as_decimal()))
}

pub fn normalize_limit_from_store(
    store: &MarketMetaStore,
    gate: &OrderGate,
    id: &MarketMetaId,
    side: MetaSide,
    price: Decimal,
    qty: Decimal,
) -> Result<(Decimal, Decimal), OrderNormalizeError> {
    let meta = store
        .get(id)
        .ok_or_else(|| OrderNormalizeError::MetaNotFound(id.clone()))?;
    normalize_limit_with_meta(gate, &meta, side, price, qty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use ucel_symbol_core::{Exchange, MarketMeta, MarketMetaId, MarketMetaSnapshot, MarketType};

    fn d(s: &str) -> Decimal {
        Decimal::from_str_exact(s).unwrap()
    }

    fn sample_meta() -> MarketMeta {
        let id = MarketMetaId::new(Exchange::Binance, MarketType::Spot, "BTC/USDT");
        let mut meta = MarketMeta::new(id, d("1"), d("0.0001"));
        meta.min_qty = Some(d("0.0001"));
        meta.max_qty = Some(d("10"));
        meta.min_notional = Some(d("100"));
        meta
    }

    #[test]
    fn normalize_limit_with_meta_applies_side_aware_rounding() {
        let gate = OrderGate::default();
        let meta = sample_meta();

        let (buy_p, buy_q) =
            normalize_limit_with_meta(&gate, &meta, MetaSide::Buy, d("1000.9"), d("0.123456"))
                .unwrap();
        assert_eq!(buy_p, d("1000"));
        assert_eq!(buy_q, d("0.1234"));

        let (sell_p, sell_q) =
            normalize_limit_with_meta(&gate, &meta, MetaSide::Sell, d("1000.1"), d("0.123456"))
                .unwrap();
        assert_eq!(sell_p, d("1001"));
        assert_eq!(sell_q, d("0.1234"));
    }

    #[test]
    fn normalize_limit_with_meta_rejects_min_notional_violation() {
        let gate = OrderGate::default();
        let meta = sample_meta();

        let err =
            normalize_limit_with_meta(&gate, &meta, MetaSide::Buy, d("100"), d("0.1")).unwrap_err();
        assert!(err.to_string().contains("min_notional"));
    }

    #[test]
    fn normalize_limit_from_store_loads_meta() {
        let gate = OrderGate::default();
        let meta = sample_meta();
        let id = meta.id.clone();

        let store = MarketMetaStore::new(Duration::from_secs(60));
        let _events = store.apply_snapshot_full(MarketMetaSnapshot::new_rest(vec![meta]));

        let (p, q) =
            normalize_limit_from_store(&store, &gate, &id, MetaSide::Buy, d("1000.9"), d("0.5"))
                .unwrap();

        assert_eq!(p, d("1000"));
        assert_eq!(q, d("0.5"));
    }
}
