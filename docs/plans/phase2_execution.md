# Phase 2: Execution（計画）

## ゴール
- Executionを「唯一の発注出口」に統合
- Intent-first / Idempotent / Reconciliation を最小セットで成立

## 主要成果物
- OrderIntentがSSOT（intent_id/client_order_id/trace_id/run_id）
- retry前照会で二重発注を防ぐ
- reconciliation（open orders/fills/balances）で矛盾を収束
- Safety（HALT/FLATTEN/CLOSE_ONLY）が全経路で強制される

## 依存
- SafetyState単一正（M1）
- AuditEvent NDJSON（M1）
