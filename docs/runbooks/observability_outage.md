# Observability Outage Runbook v1.0
（Prom targets down / Loki ingest停止）

## 目的
「監視が壊れている」状態を重大扱いし、正常に見せない。  
復旧と証跡確保を最短で行う。

## 参照（SSOT）
- 固定仕様: `docs/specs/crosscut/audit_replay_spec.md`
- 契約:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/integrity_report.schema.json`
- 運用値: `docs/policy/ucel_marketdata_thresholds.toml`（observability節）

---

## 0. 原則
- 監視欠損は "Unknown" を生む：健康に見せるのは禁止
- 監視復旧までは、Safetyは少なくとも SAFE_MODE を検討（実弾系がある場合）
- 欠損期間は必ず監査とIntegrityへ記録する

---

## 1. 初動
1) "何が落ちているか" を切り分ける  
   - Prometheus自体が落ちている / targetが落ちている  
   - Loki自体が落ちている / ingestが止まっている  
2) Support Bundle を生成（可能なら）  
   - logs_tail（アプリ側ログが生きていれば重要）
3) AuditEvent を発火  
   - `OBSERVABILITY_TARGETS_DOWN` または `OBSERVABILITY_INGEST_STOPPED`

---

## 2. 対応（Prometheus）
- Prometheusプロセスの生存確認（systemd/compose/service等）
- scrape targets の状態確認（ネットワーク/認証/ポート）
- 典型原因：
  - サービス再起動でポート変更
  - firewall/ルータ/NAT
  - Dockerネットワーク断
  - 証明書期限切れ（もしTLS使用なら）

---

## 3. 対応（Loki）
- Lokiの生存確認
- ingestの確認（promtail/fluent-bit等のforwarder含む）
- 典型原因：
  - ディスク不足
  - forwarder設定差分
  - 大量ログで圧迫（rate limit/queue）

---

## 4. 収束判定
- Prom targets が安定して UP
- Loki ingest が安定
- 欠損期間が監査イベントで記録されている
- 日次Integrityで downtimes が反映される

---

## 5. 事後
- なぜ欠損が検知できたか（検知が遅れたならアラート改善）
- "欠損中に何が起きたか" を logs_tail/audit_tail で補完
