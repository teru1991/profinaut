use tracing::{info_span, Span};

use super::logging::ObsRequiredKeys;

pub fn connection_span(ctx: &ObsRequiredKeys) -> Span {
    info_span!(
        "ucel_ws_connection",
        exchange_id = %ctx.exchange_id,
        conn_id = %ctx.conn_id,
        run_id = %ctx.run_id,
        trace_id = %ctx.trace_id,
        request_id = %ctx.request_id,
    )
}

pub fn op_span(ctx: &ObsRequiredKeys) -> Span {
    info_span!(
        "ucel_ws_operation",
        exchange_id = %ctx.exchange_id,
        conn_id = %ctx.conn_id,
        op = %ctx.op,
        symbol = %ctx.symbol,
        run_id = %ctx.run_id,
        trace_id = %ctx.trace_id,
        request_id = %ctx.request_id,
    )
}
