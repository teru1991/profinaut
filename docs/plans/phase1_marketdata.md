# Phase 1: Market Data（計画）

## ゴール
- 国内CEX Public Market Data を Golden Standard として「落とさない/戻す/証明する」を成立
- Gate（CI/Runtime/Daily）で PASS を継続

## 主要成果物
- SSOT三点セット（coverage_v2 / ws_rules / symbols）が Full
- Runtime Coverage Gate（expected vs subscribed）
- stale/gap/thin 検知と復旧（Quarantine含む）
- WAL境界の成立
- Integrity Report（日次）
- Support Bundleから再現導線が取れる

## 依存
- M1（Safety/Audit/Bundle）が先にあると事故が減る
