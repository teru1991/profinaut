use ucel_core::IdempotencyKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionAuthContext {
    pub venue: String,
    pub key_id: Option<String>,
    pub request_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionWriteIntent {
    pub symbol: String,
    pub side: String,
    pub strategy_tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCorrelation {
    pub run_id: String,
    pub request_id: String,
    pub sequence: u64,
}

pub fn derive_execution_idempotency_key(
    ctx: &ExecutionAuthContext,
    correlation: &ExecutionCorrelation,
) -> IdempotencyKey {
    let key_id = ctx
        .key_id
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());
    IdempotencyKey {
        raw: format!(
            "{}:{}:{}:{}:{}",
            ctx.venue, ctx.request_name, key_id, correlation.run_id, correlation.sequence
        ),
    }
}

pub fn derive_client_order_id(
    venue: &str,
    intent: &ExecutionWriteIntent,
    correlation: &ExecutionCorrelation,
) -> String {
    let raw = format!(
        "{}-{}-{}-{}",
        venue, intent.symbol, intent.side, correlation.sequence
    );
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_stable_idempotency_key() {
        let key = derive_execution_idempotency_key(
            &ExecutionAuthContext {
                venue: "bitbank".into(),
                key_id: Some("key1".into()),
                request_name: "place_order".into(),
            },
            &ExecutionCorrelation {
                run_id: "run1".into(),
                request_id: "req1".into(),
                sequence: 7,
            },
        );
        assert!(key.raw.contains("bitbank:place_order:key1:run1:7"));
    }
}
