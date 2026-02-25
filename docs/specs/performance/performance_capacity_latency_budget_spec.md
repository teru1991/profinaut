# Performance Engineering / Capacity / Latency Budget Core Spec v1.0（固定仕様）
Latency budgets / Throughput & backpressure / Capacity planning / Evidence-linked performance

- Document ID: PERF-CAP-LATENCY-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): T（Performance / Capacity）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Collector: `docs/specs/market_data/collector_framework_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Control Plane: `docs/specs/control_plane/control_plane_bot_manager_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
- Policy separation（固定しない）:
  - 数値閾値（ms/件数/容量/保持期間/優先度）→ `docs/policy/**`
  - チューニング手順/負荷試験手順/障害対応 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
性能は “偶然速い” ではなく、予算（budget）と制約（guard）で設計しなければ崩壊する。
本仕様は、高頻度データ・注文実行・永続化を **高速・安定・堅牢**に動かすための不変条件を固定する。

必達要件（固定）：
1) **Latency budgets exist**：重要経路は遅延予算を持ち、観測できる
2) **Hot path protection**：ホットパス（ingest/execution）を重い処理から隔離する
3) **Backpressure, no silent drop**：詰まりは制御され、黙って捨てない（integrityに出す）
4) **Capacity planning**：容量不足/IO不足は検知され、安全へ連動（SAFE）
5) **Degrade gracefully**：縮退は段階的で、P0を優先し、縮退事実を記録する
6) **Evidence-linked performance**：性能主張はメトリクス/監査/整合レポートで証拠化する
7) **Fail-safe**：観測不能（UNKNOWN）は健康扱いせず SAFE 側へ

---

## 1. 範囲（in / out）
### 1.1 In
- 遅延予算（latency budget）の枠組み
- throughput/backpressure の固定原則
- capacity（CPU/メモリ/IO/ディスク/ネット）計画の固定観点
- 縮退優先順位（P0/P1/P2）
- 性能測定の必須メトリクス・監査記録
- 性能劣化の安全連動（crosscut safety）

### 1.2 Out
- 特定ハードウェアの推奨構成（別資料で拡張可）
- 具体のチューニングパラメータ（Policy）
- 具体のベンチマーク手順詳細（Runbook）

---

## 2. Latency Budgets（固定）
### 2.1 Budget categories（固定枠組み）
各ドメインは少なくとも以下の予算を定義できる：

- Market ingest: receive → raw persist
- Canonical: raw → canonical persist
- Execution: intent → gate decision → sent → result
- Risk snapshot: input → exposure computed
- Control plane: command → state transition
- Reporting: request → (cached) response

数値（ms）は Policy、ただし「計測点と意味」は固定。

### 2.2 Measurement invariants（固定）
- 予算計測は end-to-end（E2E）と stage-by-stage の両方で可能
- trace_id/run_id と相関できる（可能な範囲で）
- 観測不能（metrics欠損）は UNKNOWN として扱う

---

## 3. Hot path protection（固定）
### 3.1 原則（固定）
- ingestion/execution のホットパスに “重い処理” を置かない
  - 大きな集計、重い解析、同期I/O、巨大フォーマット変換等
- 重い処理は非同期/別キュー/別プロセスへ
- ホットパスは bounded（メモリ/キュー深さ/CPU）であること

### 3.2 必須ガード（固定）
- キュー/バッファは上限を持ち、超えたら backpressure
- “詰まり” はメトリクスと integrity に残る（silent drop禁止）

---

## 4. Throughput & Backpressure（固定）
### 4.1 No silent drop（固定）
- データを黙って捨てない
- 取りこぼしが発生/疑われる場合は integrity_report に必ず表面化

### 4.2 Backpressure actions（固定枠組み）
バックプレッシャ時に取れる行動（実装自由だが概念固定）：
- ingestion throttle（購読数/頻度の制御）
- priority shedding（P2を縮退、P0維持）
- batching（まとめて書く）
- spill（ディスクへ退避、ただし証拠保持）
- circuit breaker（一定条件で隔離/quarantine）

縮退した事実は audit/integrity に記録される。

---

## 5. Capacity Planning（固定観点）
### 5.1 Resources（固定）
- CPU
- Memory
- Disk space
- Disk IO（IOPS/throughput/latency）
- Network bandwidth
- File descriptors / sockets
- Queue depth

### 5.2 Capacity hazards（固定）
以下は hazard として扱い crosscut safety と連動：
- disk near-full / IO stall
- persist backlog runaway
- OOM risk
- FD exhaustion
- network saturation that causes data loss risk

固定ルール：
- hazard は少なくとも SAFE へ（安全側）
- 破壊的データ操作は SAFE/EMERGENCY_STOP で拒否（crosscut）

---

## 6. Degradation policy（固定：概念）
### 6.1 Priority tiers（固定）
- P0: safety/execution integrity/ledger/critical market feeds
- P1: important analytics/secondary feeds
- P2: optional feeds/expensive derived computations

具体の分類は Policy だが、概念は固定。

### 6.2 Degrade invariants（固定）
- 縮退は段階的（P2→P1→P0の順に守る）
- 縮退中であることを “隠さない”（integrity_report, dashboard）
- 縮退解除は “緩和” に当たり危険操作になり得る（control plane）

---

## 7. Performance Evidence（固定）
### 7.1 Required metrics（固定カテゴリ）
最低限、以下を観測できる：
- E2E latency（各budgetカテゴリ）
- stage latency（receive→persist 等）
- throughput（msg/s, orders/s, bytes/s）
- backlog/queue depth
- backpressure events count/duration
- drops prevented / duplicates suppressed
- IO metrics（disk usage, IO latency）
- error rates（kind別：429/auth/protocol/integrity）
- quarantine counts/durations
- kill-switch level time series

### 7.2 Evidence chain（固定）
性能異常時に少なくとも以下へ辿れる：
- audit_event（backpressure/quarantine/kill-switch）
- integrity_report（欠損/遅延/unknown）
- gate_results（UNKNOWN/FAIL）
- support bundle（必要時）

---

## 8. Safety coupling（固定）
- 観測不能（UNKNOWN）や capacity hazard は gate UNKNOWN/FAIL へ
- それにより safety_mode は SAFE 側へ（最低限）
- “速いけど嘘/欠損” を許さない（integrityが優先）

---

## 9. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `perf.budget.violation`（which budget + window）
- `perf.backpressure.start/end`（scope + reason）
- `perf.capacity.hazard`（disk/IO/FD/OOM risk）
- `perf.degrade.applied`（P2 shedding等）
- `perf.degrade.recovered`
- `support_bundle.created`（必要時）
- `integrity.record` / `gate.record`

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) 遅延予算の計測点が存在し、E2E latency が観測できる
2) backpressure が機能し、silent drop しない（欠損は integrity に出る）
3) capacity hazard（disk/IO等）で SAFE 側へ寄る
4) P0が優先され、P2が縮退される（縮退事実が記録される）
5) 性能異常の証拠（audit/integrity/gate/bundle）が辿れる
6) 監視欠損が UNKNOWN として扱われ、成功扱いされない

---

## 11. Policy/Runbookへ逃がす点
- 具体のms/閾値/容量/保持期間/優先度分類
- 具体のチューニング・負荷試験手順・復旧手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
