use tracing::{error, info, info_span, warn, Span};
use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsRequiredKeys {
    pub exchange_id: String,
    pub conn_id: String,
    pub op: String,
    pub symbol: String,
    pub run_id: String,
    pub trace_id: String,
    pub request_id: String,
}

impl ObsRequiredKeys {
    pub fn try_new(
        exchange_id: impl Into<String>,
        conn_id: impl Into<String>,
        op: impl Into<String>,
        symbol: impl Into<String>,
        run_id: impl Into<String>,
        trace_id: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Result<Self, UcelError> {
        let keys = Self {
            exchange_id: exchange_id.into(),
            conn_id: conn_id.into(),
            op: op.into(),
            symbol: symbol.into(),
            run_id: run_id.into(),
            trace_id: trace_id.into(),
            request_id: request_id.into(),
        };
        ensure_required_fields(&keys)?;
        Ok(keys)
    }

    pub fn try_new_wildcard_symbol(
        exchange_id: impl Into<String>,
        conn_id: impl Into<String>,
        op: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Result<Self, UcelError> {
        Self::try_new(
            exchange_id,
            conn_id,
            op,
            "*",
            run_id,
            "trace-unknown",
            "request-unknown",
        )
    }
}

pub fn ensure_required_fields(ctx: &ObsRequiredKeys) -> Result<(), UcelError> {
    fn ok(v: &str) -> bool {
        !v.trim().is_empty()
    }
    if [
        ok(&ctx.exchange_id),
        ok(&ctx.conn_id),
        ok(&ctx.op),
        ok(&ctx.symbol),
        ok(&ctx.run_id),
        ok(&ctx.trace_id),
        ok(&ctx.request_id),
    ]
    .into_iter()
    .all(|x| x)
    {
        return Ok(());
    }
    Err(UcelError::new(
        ErrorCode::CatalogInvalid,
        "observability required keys must be non-empty (exchange_id/conn_id/op/symbol/run_id/trace_id/request_id)",
    ))
}

pub fn span_required(_name: &'static str, k: &ObsRequiredKeys) -> Span {
    info_span!(
        "ucel_transport",
        exchange_id = %k.exchange_id,
        conn_id = %k.conn_id,
        op = %k.op,
        symbol = %k.symbol,
        run_id = %k.run_id,
        trace_id = %k.trace_id,
        request_id = %k.request_id,
    )
}

pub fn info_with_ctx(ctx: &ObsRequiredKeys, message: &str) {
    info!(exchange_id=%ctx.exchange_id, conn_id=%ctx.conn_id, op=%ctx.op, symbol=%ctx.symbol, run_id=%ctx.run_id, trace_id=%ctx.trace_id, request_id=%ctx.request_id, "{message}");
}

pub fn warn_with_ctx(ctx: &ObsRequiredKeys, reason: &str, message: &str) {
    warn!(exchange_id=%ctx.exchange_id, conn_id=%ctx.conn_id, op=%ctx.op, symbol=%ctx.symbol, run_id=%ctx.run_id, trace_id=%ctx.trace_id, request_id=%ctx.request_id, reason=%reason, "{message}");
}

pub fn error_with_ctx(ctx: &ObsRequiredKeys, reason: &str, message: &str) {
    error!(exchange_id=%ctx.exchange_id, conn_id=%ctx.conn_id, op=%ctx.op, symbol=%ctx.symbol, run_id=%ctx.run_id, trace_id=%ctx.trace_id, request_id=%ctx.request_id, reason=%reason, "{message}");
}
