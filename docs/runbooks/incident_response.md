# Incident Response Runbook v1.0

## 原則
- まず安全：必要なら `HALT`（最優先）
- 次に観測：監視欠損があれば "見えてない" を明示する
- 次に証跡：AuditEvent と Support Bundle を確保する
- 最後に復旧：原因を分類して段階復旧

---

## 1. 即時対応（最短）
1) ダッシュボード/CLI から **HALT** を発動（SafetyState）  
2) `support_bundle` を生成（manifest + safety_state + audit_tail + logs_tail）  
3) 監視欠損（Prom targets down / Loki ingest停止）を確認  
4) Executionなら「外部混入/照合乖離」有無を確認  
5) Market Dataなら stale/gap/thin と Quarantine を確認

---

## 2. 典型原因カテゴリ（分類）
- ExecutionAnomaly
- ReconciliationDivergence（外部混入含む）
- MarketDataIntegrity（gap/Unknown急増）
- LatencyBackpressure（WAL/IO/queue）
- ClockTime（drift/逆行）
- SecretsSecurity（漏洩検知）
- Observability欠損

---

## 3. 復旧の段階
- Step A：観測復旧（Prom/Loki/Grafana）
- Step B：Collector復旧（再接続/再購読/Quarantine解除条件確認）
- Step C：Execution復旧（照合→安全縮退解除の順）
- Step D：HALT解除（強操作：理由・期限・監査必須）

---

## 4. 事後（必須）
- 重大監査イベントの整理（type別）
- 再発防止（Policy閾値調整 or 実装修正）
- 必要ならリプレイで再現確認
