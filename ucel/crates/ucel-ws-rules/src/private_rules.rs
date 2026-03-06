use crate::model::ExchangeWsRules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckMode {
    Explicit,
    ImplicitObservation,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateRuleView {
    pub auth_required: bool,
    pub ack_mode: AckMode,
    pub supports_reauth: bool,
}

pub fn private_rule_view(exchange: &str, rules: &ExchangeWsRules) -> PrivateRuleView {
    let lower = exchange.to_ascii_lowercase();
    let ack_mode = match lower.as_str() {
        "coincheck" => AckMode::ImplicitObservation,
        "sbivc" | "upbit" => AckMode::None,
        _ => AckMode::Explicit,
    };
    let supports_reauth = matches!(lower.as_str(), "bitbank" | "bitflyer" | "gmocoin");
    PrivateRuleView {
        auth_required: true,
        ack_mode,
        supports_reauth: supports_reauth && rules.entitlement.is_some(),
    }
}
