use ucel_core::EquityCorporateAction;

pub fn sort_actions(mut actions: Vec<EquityCorporateAction>) -> Vec<EquityCorporateAction> {
    actions.sort_by_key(|a| match a {
        EquityCorporateAction::Split { effective_date, .. } => effective_date.clone(),
        EquityCorporateAction::ReverseSplit { effective_date, .. } => effective_date.clone(),
        EquityCorporateAction::Dividend { ex_date, .. } => ex_date.clone(),
        EquityCorporateAction::SymbolChange { effective_date, .. } => effective_date.clone(),
        EquityCorporateAction::Delist { effective_date, .. } => effective_date.clone(),
    });
    actions
}

pub fn merge_actions(
    a: Vec<EquityCorporateAction>,
    b: Vec<EquityCorporateAction>,
) -> Vec<EquityCorporateAction> {
    sort_actions([a, b].concat())
}
