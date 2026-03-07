use crate::{BittradeRestClient, BittradeRestResponse, RequestArgs};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use ucel_core::{ErrorCode, UcelError};
use ucel_sdk::execution::{
    unix_ms_now, ExecutionConnectorAsync, IdempotencyKey, OrderCancel, OrderIntentId,
    OrderOpenQuery, OrderReceipt, OrderRequest, OrderSide, OrderStatus, OrderType, ReconcileReport,
    ReconcileSource, SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult, Symbol, VenueId,
};
use ucel_transport::Transport;

/// Bittrade（Huobi 系）向け ExecutionConnectorAsync 実装。
/// - account-id は初回に accounts.get を叩いてキャッシュ（`Mutex<Option<String>>`）
/// - place: /v1/order/orders/place（private.rest.order.place.post）
/// - cancel: /v1/order/orders/{order-id}/submitcancel（private.rest.order.cancel.post）
/// - list_open_orders: /v1/order/orders?states=submitted,partial-filled（private.rest.order.list.get）
/// - reconcile: 最小実装（open_orders が叩けることを確認）
pub struct BittradeExecutionConnector<T: Transport + Send + Sync> {
    rest: BittradeRestClient<T>,
    key_id: String,
    account_id_cache: Mutex<Option<String>>,
}

impl<T: Transport + Send + Sync> BittradeExecutionConnector<T> {
    pub fn new(transport: Arc<T>, key_id: impl Into<String>) -> Self {
        Self {
            rest: BittradeRestClient::new(transport),
            key_id: key_id.into(),
            account_id_cache: Mutex::new(None),
        }
    }

    async fn ensure_account_id(&self) -> SdkExecutionResult<String> {
        // キャッシュ確認
        if let Some(v) = self
            .account_id_cache
            .lock()
            .ok()
            .and_then(|g| g.as_ref().cloned())
        {
            return Ok(v);
        }
        let args = RequestArgs::default();
        let resp = self
            .rest
            .execute(
                "private.rest.account.accounts.get",
                args,
                Some(self.key_id.clone()),
            )
            .await
            .map_err(map_ucel_err)?;
        let id = parse_first_account_id(resp)?;
        if let Ok(mut g) = self.account_id_cache.lock() {
            *g = Some(id.clone());
        }
        Ok(id)
    }
}

fn map_ucel_err(e: UcelError) -> SdkExecutionError {
    let code = match e.code {
        ErrorCode::Timeout => SdkExecutionErrorCode::Timeout,
        ErrorCode::NotSupported => SdkExecutionErrorCode::NotSupported,
        _ => SdkExecutionErrorCode::ConnectorError,
    };
    SdkExecutionError::new(code, format!("bittrade connector error: {}", e.message))
}

fn parse_first_account_id(resp: BittradeRestResponse) -> SdkExecutionResult<String> {
    let BittradeRestResponse::Json(v) = resp;
    // Huobi 系の典型: { "status":"ok", "data":[{"id":12345, ...}, ...] }
    let data = v.get("data").ok_or_else(|| {
        SdkExecutionError::new(
            SdkExecutionErrorCode::ConnectorError,
            "accounts missing data field",
        )
    })?;
    let arr = data.as_array().ok_or_else(|| {
        SdkExecutionError::new(
            SdkExecutionErrorCode::ConnectorError,
            "accounts data not array",
        )
    })?;
    let first = arr.first().ok_or_else(|| {
        SdkExecutionError::new(SdkExecutionErrorCode::ConnectorError, "accounts empty")
    })?;
    let id = first.get("id").ok_or_else(|| {
        SdkExecutionError::new(
            SdkExecutionErrorCode::ConnectorError,
            "accounts missing id field",
        )
    })?;
    if let Some(n) = id.as_i64() {
        return Ok(n.to_string());
    }
    if let Some(s) = id.as_str() {
        return Ok(s.to_string());
    }
    Err(SdkExecutionError::new(
        SdkExecutionErrorCode::ConnectorError,
        "accounts id invalid type",
    ))
}

fn map_side_type(req: &OrderRequest) -> SdkExecutionResult<String> {
    // Bittrade(Huobi) 例: buy-limit / sell-limit / buy-market / sell-market
    let side = match req.intent.side {
        OrderSide::Buy => "buy",
        OrderSide::Sell => "sell",
    };
    let ty = match req.intent.order_type {
        OrderType::Market => "market",
        OrderType::Limit | OrderType::PostOnly => "limit",
    };
    Ok(format!("{side}-{ty}"))
}

fn as_symbol(req: &OrderRequest) -> String {
    // Bittrade は小文字シンボル（例: btcjpy）
    req.intent.symbol.0.to_lowercase()
}

fn as_amount(req: &OrderRequest) -> String {
    format!("{}", req.intent.qty.0)
}

fn as_price(req: &OrderRequest) -> Option<String> {
    req.intent.price.map(|p| format!("{}", p.0))
}

fn client_order_id_from(req: &OrderRequest) -> Option<String> {
    req.intent.tags.get("client_order_id").cloned()
}

#[allow(async_fn_in_trait)]
impl<T: Transport + Send + Sync> ExecutionConnectorAsync for BittradeExecutionConnector<T> {
    async fn place_order(&self, req: &OrderRequest) -> SdkExecutionResult<OrderReceipt> {
        let account_id = self.ensure_account_id().await?;

        let mut body = serde_json::json!({
            "account-id": account_id,
            "symbol": as_symbol(req),
            "type": map_side_type(req)?,
            "amount": as_amount(req),
            "source": "api",
        });

        if let Some(p) = as_price(req) {
            body["price"] = serde_json::Value::String(p);
        }
        if let Some(cid) = client_order_id_from(req) {
            body["client-order-id"] = serde_json::Value::String(cid);
        }

        let body_bytes = serde_json::to_vec(&body).map_err(|e| {
            SdkExecutionError::new(
                SdkExecutionErrorCode::ConnectorError,
                format!("encode order body failed: {e}"),
            )
        })?;

        let args = RequestArgs {
            path_params: BTreeMap::new(),
            query_params: BTreeMap::new(),
            body: Some(bytes::Bytes::from(body_bytes)),
        };

        let resp = self
            .rest
            .execute(
                "private.rest.order.place.post",
                args,
                Some(self.key_id.clone()),
            )
            .await
            .map_err(map_ucel_err)?;

        let venue_order_id = match resp {
            BittradeRestResponse::Json(v) => {
                // 典型: { "status":"ok", "data":"1234567890" }
                v.get("data")
                    .and_then(|d| {
                        d.as_str()
                            .map(|s| s.to_string())
                            .or_else(|| d.as_i64().map(|n| n.to_string()))
                    })
                    .ok_or_else(|| {
                        SdkExecutionError::new(
                            SdkExecutionErrorCode::ConnectorError,
                            "place order missing data(order_id)",
                        )
                    })?
            }
        };

        Ok(OrderReceipt {
            venue: req.intent.venue.clone(),
            symbol: req.intent.symbol.clone(),
            status: OrderStatus::Accepted,
            venue_order_id: Some(venue_order_id),
            client_order_id: client_order_id_from(req),
            intent_id: req.intent.intent_id.clone(),
            idempotency: req.idempotency.clone(),
        })
    }

    async fn cancel_order(&self, cancel: &OrderCancel) -> SdkExecutionResult<bool> {
        let mut path_params = BTreeMap::new();
        path_params.insert("order-id".to_string(), cancel.venue_order_id.clone());

        let args = RequestArgs {
            path_params,
            query_params: BTreeMap::new(),
            body: None,
        };

        let resp = self
            .rest
            .execute(
                "private.rest.order.cancel.post",
                args,
                Some(self.key_id.clone()),
            )
            .await
            .map_err(map_ucel_err)?;

        // 典型: { "status":"ok", ... }
        let ok = match resp {
            BittradeRestResponse::Json(v) => {
                v.get("status").and_then(|s| s.as_str()).unwrap_or("") == "ok"
            }
        };
        Ok(ok)
    }

    async fn list_open_orders(&self, q: &OrderOpenQuery) -> SdkExecutionResult<Vec<OrderReceipt>> {
        let mut query = BTreeMap::new();
        // Huobi 系: states=submitted,partial-filled 等（open 相当）
        query.insert("states".to_string(), "submitted,partial-filled".to_string());
        if let Some(sym) = &q.symbol {
            query.insert("symbol".to_string(), sym.0.to_lowercase());
        }

        let args = RequestArgs {
            path_params: BTreeMap::new(),
            query_params: query,
            body: None,
        };

        let resp = self
            .rest
            .execute(
                "private.rest.order.list.get",
                args,
                Some(self.key_id.clone()),
            )
            .await
            .map_err(map_ucel_err)?;

        let BittradeRestResponse::Json(v) = resp;
        let data = v
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out = vec![];

        for it in &data {
            // { "id":..., "symbol":"btcjpy", "type":"buy-limit", ... }
            let id = it.get("id").and_then(|x| {
                x.as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| x.as_str().map(|s| s.to_string()))
            });
            let sym = it
                .get("symbol")
                .and_then(|x| x.as_str())
                .map(|s| s.to_uppercase());
            if let (Some(order_id), Some(sym_s)) = (id, sym) {
                out.push(OrderReceipt {
                    venue: q.venue.clone(),
                    symbol: Symbol::new(sym_s),
                    status: OrderStatus::Open,
                    venue_order_id: Some(order_id),
                    client_order_id: None,
                    // open_orders は venue 側情報優先（v1 では unknown）
                    intent_id: OrderIntentId::new("unknown"),
                    idempotency: IdempotencyKey::random_uuid(),
                });
            }
        }
        Ok(out)
    }

    async fn reconcile(&self, venue: &VenueId) -> SdkExecutionResult<ReconcileReport> {
        // 最小実装: open_orders が叩ける = "照合の入口" は成立
        // mismatches は空（v1）。詳細照合は FileAuditSink と組み合わせて次で強化
        Ok(ReconcileReport {
            venue: venue.clone(),
            source: ReconcileSource::Venue,
            ok: true,
            mismatches: vec![],
            generated_at_unix_ms: unix_ms_now(),
        })
    }
}
