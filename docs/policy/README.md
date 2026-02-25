# docs/policy（運用値 / 可変）

このディレクトリは **Policy（運用値）** を管理します。  
Core Spec（固定仕様）を変更せず、環境や運用で調整したい「値」だけをここに集約します。

## ルール
- Policyは **契約（docs/contracts）** と **固定仕様（docs/specs）** を破壊しない
- 値の変更は許容されるが、変更理由と影響（何が変わるか）を記録する
- Plan/RunbookはPolicyを参照して運用する

## 代表例
- 閾値（SLO、stale、gap、thin、WAL、disk、clock、obs欠損など）
- 禁止キー（redaction）
- 保持/圧縮/退避（retention）
- リモート閲覧（Cloudflare Access/Grafana）

## Policy Index

### 既存ポリシー
- `forbidden_keys.toml`
  - 目的: ログ/出力時の禁止キー（秘匿情報）制御
  - 対応Spec: [security_hardening_threat_incident_spec](../specs/security/security_hardening_threat_incident_spec.md), [storage_integrity_spec](../specs/storage/storage_integrity_spec.md)
  - 主要キー: `forbidden_keys`, `strict_mode`, `redaction`
- `remote_access.toml`
  - 目的: リモート閲覧・運用アクセス条件
  - 対応Spec: [identity_access_spec](../specs/security/identity_access_spec.md), [control_plane_bot_manager_spec](../specs/control_plane/control_plane_bot_manager_spec.md)
  - 主要キー: `access`, `session`, `network`
- `retention.toml`
  - 目的: データ保持・圧縮・削除運用値
  - 対応Spec: [storage_integrity_spec](../specs/storage/storage_integrity_spec.md), [data_catalog_lineage_spec](../specs/data_governance/data_catalog_lineage_spec.md)
  - 主要キー: `retention`, `compaction`, `archive`
- `ucel_marketdata_thresholds.toml`
  - 目的: 市場データ品質しきい値（UCEL）
  - 対応Spec: [collector_framework_spec](../specs/market_data/collector_framework_spec.md), [observability_slo_diagnostics_spec](../specs/observability/observability_slo_diagnostics_spec.md)
  - 主要キー: `thresholds`, `staleness`, `gap`

### DOC-POLICY-PACK-001 追加ポリシー
- `execution_limits.toml`
  - 目的: 実行系の安全上限（レート・サイズ・価格乖離・タイムアウト）
  - 対応Spec: [runtime_execution_safety_spec](../specs/execution/runtime_execution_safety_spec.md), [order_trade_ledger_pnl_spec](../specs/accounting/order_trade_ledger_pnl_spec.md)
  - 主要キー: `rate_limit`, `order_size`, `price_sanity`, `timeouts`, `retry`
- `risk_limits.toml`
  - 目的: ポートフォリオ/損失/UNKNOWN時挙動の上限
  - 対応Spec: [portfolio_risk_management_spec](../specs/risk/portfolio_risk_management_spec.md), [runtime_execution_safety_spec](../specs/execution/runtime_execution_safety_spec.md)
  - 主要キー: `limits`, `loss_controls`, `unknown_handling`
- `control_plane_authz_matrix.toml`
  - 目的: Control Plane RBAC と break-glass 制約
  - 対応Spec: [control_plane_bot_manager_spec](../specs/control_plane/control_plane_bot_manager_spec.md), [identity_access_spec](../specs/security/identity_access_spec.md)
  - 主要キー: `roles`, `permissions`, `break_glass`
- `strategy_runtime_constraints.toml`
  - 目的: Strategy runtime sandbox 制約と隔離条件
  - 対応Spec: [bot_sdk_plugin_boundary_spec](../specs/strategy_runtime/bot_sdk_plugin_boundary_spec.md), [runtime_execution_safety_spec](../specs/execution/runtime_execution_safety_spec.md)
  - 主要キー: `limits`, `quarantine`
- `observability_slo_alerts.toml`
  - 目的: SLO目標とアラート閾値
  - 対応Spec: [observability_slo_diagnostics_spec](../specs/observability/observability_slo_diagnostics_spec.md), [performance_capacity_latency_budget_spec](../specs/performance/performance_capacity_latency_budget_spec.md)
  - 主要キー: `slo`, `alerts`, `notifications`
- `support_bundle_triggers.toml`
  - 目的: サポートバンドル自動生成トリガー
  - 対応Spec: [observability_slo_diagnostics_spec](../specs/observability/observability_slo_diagnostics_spec.md), [automation_scheduling_routine_ops_spec](../specs/automation/automation_scheduling_routine_ops_spec.md)
  - 主要キー: `triggers`, `limits`
- `automation_jobs.toml`
  - 目的: 定期自動ジョブの実行窓/有効化設定
  - 対応Spec: [automation_scheduling_routine_ops_spec](../specs/automation/automation_scheduling_routine_ops_spec.md), [storage_integrity_spec](../specs/storage/storage_integrity_spec.md)
  - 主要キー: `jobs.integrity_daily`, `jobs.gate_daily`, `jobs.retention_compaction`
- `performance_budgets.toml`
  - 目的: レイテンシ予算と容量危険域しきい値
  - 対応Spec: [performance_capacity_latency_budget_spec](../specs/performance/performance_capacity_latency_budget_spec.md), [collector_framework_spec](../specs/market_data/collector_framework_spec.md)
  - 主要キー: `latency_budget`, `capacity_hazards`
- `testing_quality_gates.toml`
  - 目的: CIの品質ゲートとカバレッジ下限
  - 対応Spec: [testing_qa_contract_chaos_spec](../specs/testing/testing_qa_contract_chaos_spec.md), [backtest_forwardtest_repro_spec](../specs/research_testing/backtest_forwardtest_repro_spec.md)
  - 主要キー: `gates`, `coverage`
- `onchain_finality_rpc_policy.toml`
  - 目的: オンチェーンFinality/RPC整合性閾値
  - 対応Spec: [onchain_ingestion_finality_reorg_integrity_spec](../specs/onchain/onchain_ingestion_finality_reorg_integrity_spec.md), [storage_integrity_spec](../specs/storage/storage_integrity_spec.md)
  - 主要キー: `finality`, `rpc`
- `defi_dex_quote_mev_policy.toml`
  - 目的: DEX見積制約とMEVリスク受容上限
  - 対応Spec: [defi_dex_analytics_arbitrage_spec](../specs/defi_dex/defi_dex_analytics_arbitrage_spec.md), [runtime_execution_safety_spec](../specs/execution/runtime_execution_safety_spec.md)
  - 主要キー: `quote`, `mev`
- `fx_macro_sources_policy.toml`
  - 目的: FX/Macroの優先ソースと鮮度上限
  - 対応Spec: [fx_macro_ingestion_normalization_spec](../specs/fx_macro/fx_macro_ingestion_normalization_spec.md), [collector_framework_spec](../specs/market_data/collector_framework_spec.md)
  - 主要キー: `sources`, `staleness`
- `equities_ir_sources_scoring_policy.toml`
  - 目的: Equities IR ソース優先順位とスコア重み
  - 対応Spec: [equities_ir_financials_ingestion_scoring_spec](../specs/equities_ir/equities_ir_financials_ingestion_scoring_spec.md), [collector_framework_spec](../specs/market_data/collector_framework_spec.md)
  - 主要キー: `sources`, `scoring`
- `reporting_export_policy.toml`
  - 目的: レポート出力形式と安全制約
  - 対応Spec: [reporting_dashboard_explainability_spec](../specs/reporting/reporting_dashboard_explainability_spec.md), [tax_compliance_legal_reporting_spec](../specs/tax_compliance/tax_compliance_legal_reporting_spec.md)
  - 主要キー: `export`
- `tax_regime_policy.toml`
  - 目的: 税制レジームと計算方式選択
  - 対応Spec: [tax_compliance_legal_reporting_spec](../specs/tax_compliance/tax_compliance_legal_reporting_spec.md), [tax_compliance_legal_reporting_spec (governance)](../specs/governance/tax_compliance_legal_reporting_spec.md)
  - 主要キー: `regime`, `methods`
- `i18n_locale_policy.toml`
  - 目的: 多市場対応のデフォルト言語/通貨/タイムゾーン
  - 対応Spec: [i18n_localization_multimarket_readiness_spec](../specs/i18n/i18n_localization_multimarket_readiness_spec.md), [reporting_dashboard_explainability_spec](../specs/reporting/reporting_dashboard_explainability_spec.md)
  - 主要キー: `defaults`
- `ux_safety_ui_policy.toml`
  - 目的: 危険操作UIの確認強度と文言
  - 対応Spec: [ux_human_factors_safety_ui_patterns_spec](../specs/ux_safety/ux_human_factors_safety_ui_patterns_spec.md), [control_plane_bot_manager_spec](../specs/control_plane/control_plane_bot_manager_spec.md)
  - 主要キー: `confirm`
- `governance_release_policy.toml`
  - 目的: リリース許可窓と承認人数
  - 対応Spec: [tax_compliance_legal_reporting_spec (governance)](../specs/governance/tax_compliance_legal_reporting_spec.md), [control_plane_bot_manager_spec](../specs/control_plane/control_plane_bot_manager_spec.md)
  - 主要キー: `release`, `approvals`
- `security_incident_policy.toml`
  - 目的: インシデント封じ込め既定動作と閾値
  - 対応Spec: [security_hardening_threat_incident_spec](../specs/security/security_hardening_threat_incident_spec.md), [identity_access_spec](../specs/security/identity_access_spec.md)
  - 主要キー: `containment`, `thresholds`
- `data_catalog_access_policy.toml`
  - 目的: データセット分類と輸出制約
  - 対応Spec: [data_catalog_lineage_spec](../specs/data_governance/data_catalog_lineage_spec.md), [reporting_dashboard_explainability_spec](../specs/reporting/reporting_dashboard_explainability_spec.md)
  - 主要キー: `classes`, `export`

## UCEL Golden Policy
- Canonical: docs/policy/ucel_golden_policy_pack.md
- Normalized TOML index: docs/policy/ucel_golden/README.md
