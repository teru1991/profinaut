use tracing::{info_span, Span};
use ucel_core::{ErrorCode, UcelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsRequiredKeys {
    pub exchange_id: String,
    pub conn_id: String,
    pub op: String,
    pub symbol: String,
    pub run_id: String,
}

impl ObsRequiredKeys {
    pub fn try_new(
        exchange_id: impl Into<String>,
        conn_id: impl Into<String>,
        op: impl Into<String>,
        symbol: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Result<Self, UcelError> {
        let exchange_id = exchange_id.into();
        let conn_id = conn_id.into();
        let op = op.into();
        let symbol = symbol.into();
        let run_id = run_id.into();

        fn ok(s: &str) -> bool {
            !s.trim().is_empty()
        }

        if !ok(&exchange_id) || !ok(&conn_id) || !ok(&op) || !ok(&symbol) || !ok(&run_id) {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                "observability required keys must be non-empty (exchange_id/conn_id/op/symbol/run_id)",
            ));
        }

        Ok(Self {
            exchange_id,
            conn_id,
            op,
            symbol,
            run_id,
        })
    }

    pub fn try_new_wildcard_symbol(
        exchange_id: impl Into<String>,
        conn_id: impl Into<String>,
        op: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Result<Self, UcelError> {
        Self::try_new(exchange_id, conn_id, op, "*", run_id)
    }
}

pub fn span_required(_name: &'static str, k: &ObsRequiredKeys) -> Span {
    info_span!(
        "ucel_transport",
        exchange_id = %k.exchange_id,
        conn_id = %k.conn_id,
        op = %k.op,
        symbol = %k.symbol,
        run_id = %k.run_id
    )
}
