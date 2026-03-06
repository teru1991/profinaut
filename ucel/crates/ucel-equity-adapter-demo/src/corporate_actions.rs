use crate::errors::{EquityAdapterError, EquityAdapterErrorKind};
use crate::symbols::resolve_symbol;
use crate::DemoEquityAdapter;
use ucel_core::{EquityCorporateAction, EquityDividend};
use ucel_equity_core::corporate_actions::sort_actions;

pub fn get_corporate_actions(
    adapter: &DemoEquityAdapter,
    symbol: &str,
    _from: &str,
    _to: &str,
) -> Result<Vec<EquityCorporateAction>, EquityAdapterError> {
    let resolved = resolve_symbol(adapter, symbol)?;
    let endpoint = format!("/actions/{}", resolved.vendor_symbol);
    let raw = adapter.http.get_json(&endpoint)?;
    let arr = raw.as_array().ok_or_else(|| {
        EquityAdapterError::new(
            EquityAdapterErrorKind::CorporateActionUnavailable,
            "actions not array",
        )
    })?;
    let mut out = Vec::new();
    for a in arr {
        if a.get("type").and_then(|v| v.as_str()).unwrap_or_default() == "dividend" {
            out.push(EquityCorporateAction::Dividend {
                symbol: resolved.clone(),
                ex_date: a
                    .get("ex_date")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                dividend: EquityDividend {
                    cash_amount: a.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    currency: a
                        .get("currency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("USD")
                        .to_string(),
                },
            });
        }
    }
    Ok(sort_actions(out))
}
