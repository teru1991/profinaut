use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VenueId(pub String);

impl VenueId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderIntentId(pub String);

impl OrderIntentId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// 価格・数量は "Decimal SSOT" が既にある前提だが、ここでは public surface 固定が目的なので
/// 既存の Decimal 型がある場合はそちらへ差し替える（この型名を re-export し続ける想定）。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Price(pub f64);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantity(pub f64);

/// Execution-surface 固有の発注方向（execution::OrderSide）。
/// ucel_symbol_core::OrderSide とは別型。今後の統合タスクで alias/From を追加できる。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Execution-surface 固有の注文種別（execution::OrderType）。
/// ucel_core::OrderType とは別型。互換変換は次タスクで。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    PostOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderTimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// 純粋シミュレーション。外部発注を絶対にしない。
    Paper,
    /// 実発注はしないが、実系 API で validate/quote/constraints を照合して監査に残す。
    Shadow,
    /// 実発注。冪等・監査・照合が必須。
    Live,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Accepted,
    Rejected,
    Open,
    PartiallyFilled,
    Filled,
    Canceled,
    Expired,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderIntent {
    pub intent_id: OrderIntentId,
    pub venue: VenueId,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub tif: Option<OrderTimeInForce>,
    pub price: Option<Price>,
    pub qty: Quantity,
    /// client_order_id 等の "外部へ透過させる" 追加フィールドはここで保持。
    /// venue コネクタ側が必要に応じて利用する。
    pub tags: std::collections::BTreeMap<String, String>,
}

impl OrderIntent {
    pub fn validate_basic(&self) -> Result<(), &'static str> {
        if self.venue.0.trim().is_empty() {
            return Err("venue empty");
        }
        if self.symbol.0.trim().is_empty() {
            return Err("symbol empty");
        }
        if !self.qty.0.is_finite() || self.qty.0 <= 0.0 {
            return Err("qty invalid");
        }
        if matches!(self.order_type, OrderType::Limit | OrderType::PostOnly) {
            let p = self.price.ok_or("price required for limit/postonly")?;
            if !p.0.is_finite() || p.0 <= 0.0 {
                return Err("price invalid");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderRequest {
    pub mode: ExecutionMode,
    pub intent: OrderIntent,
    pub idempotency: crate::execution::IdempotencyKey,
    /// "入口は ucel-sdk だけ" を守るため、ここで監査用 run_id 等を受けられるようにする。
    pub run_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderReceipt {
    pub venue: VenueId,
    pub symbol: Symbol,
    pub status: OrderStatus,
    /// venue が返す order_id（取引所注文ID）
    pub venue_order_id: Option<String>,
    /// クライアント側が指定する client_order_id があるなら格納
    pub client_order_id: Option<String>,
    /// 監査用：入口で生成した intent_id/idempotency を必ず返す
    pub intent_id: OrderIntentId,
    pub idempotency: crate::execution::IdempotencyKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderCancel {
    pub venue: VenueId,
    pub symbol: Symbol,
    pub venue_order_id: String,
    pub idempotency: crate::execution::IdempotencyKey,
    pub run_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderOpenQuery {
    pub venue: VenueId,
    pub symbol: Option<Symbol>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub receipt: OrderReceipt,
    pub audit_event_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReconcileSource {
    /// 取引所の REST/WS などから得た "実データ"
    Venue,
    /// ローカルの監査ログ（過去の発注・キャンセル・照合結果）
    AuditLog,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconcileReport {
    pub venue: VenueId,
    pub source: ReconcileSource,
    pub ok: bool,
    pub mismatches: Vec<String>,
    pub generated_at_unix_ms: u64,
}

impl fmt::Display for VenueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
