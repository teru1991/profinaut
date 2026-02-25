# Observability / SLO / Alerts / Diagnostics Core Spec v1.0（固定仕様）
Metrics / Logs / Traces / SLO / Alerting / Support Bundle Triggers

- Document ID: OBS-SLO-DIAG-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): F（Observability）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Collector: `docs/specs/market_data/collector_framework_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
- Policy separation（固定しない）:
  - SLO目標値、アラート閾値、通知経路、抑制期間、bundle生成条件 → `docs/policy/**`
  - オンコール/復旧手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、全システムに対して「観測が正しく、欠損が隠れず、証拠が残り、安全に連動する」ことを固定保証する。

必達要件（固定）：
1) **Observability honesty**：監視欠損は “健康” ではなく “不明（UNKNOWN）”
2) **Safety coupling**：UNKNOWN は crosscut safety により SAFE 側へ倒れる（最低限）
3) **Evidence-first**：SLO/アラートの根拠が audit/integrity/replay で追える
4) **Low overhead hot path**：実行・収集のホットパスを潰さない（非同期/バッチ/集計）
5) **No secret leakage**：ログ/メトリクス/トレース/バンドルに秘密を入れない
6) **Actionable**：アラートは「原因→観測→自動復旧→手動介入」へ接続できる

---

## 1. 観測の責務境界（in / out）
### 1.1 In（対象）
- Metrics/Logs/Traces の必須項目（共通規約）
- 監視欠損の扱い（UNKNOWN）
- SLO/SLI の定義（固定枠組み）
- Alerting の分類（固定枠組み）
- Support Bundle の生成トリガ枠組み（固定）
- 安全（Safety Mode）との連動（固定）

### 1.2 Out（対象外）
- Prometheus/Loki/Grafana 等の具体製品の設定詳細（runbooks/policy）
- 通知ツール（Slack等）の実装
- UI設計（dashboard domain）

---

## 2. 共通観測規約（固定）
Platform Foundation の共通ID体系に従い、最低限以下を相関可能にする：

- `trace_id` / `run_id`（可能な範囲で）
- `component`（collector/execution/storage/controlplane等）
- `instance_id`
- `schema_version`（構造化イベント系）
- `venue` / `stream_id` / `symbol`（該当する場合）
- `actor`（人/サービス：危険操作系）

固定ルール：
- これらが欠落するログ/メトリクスが増えること自体が品質低下として扱われる（Policyで閾値）

---

## 3. Logs（固定）
### 3.1 構造化ログ（固定）
- JSON構造を基本とし、最低限：
  - timestamp (UTC)
  - level
  - component
  - run_id
  - trace_id（可能なら）
  - message
  - error.kind / error.code（Standard Error Model 準拠）
を持つ。

### 3.2 Redaction（固定）
- 禁止キー検知（forbidden-key scan）をログ出力前に適用できる
- 検知した場合：
  - ログ出力は “赤塗り済み” のみ
  - audit_event に `secret.guard.triggered` を出す（秘密値は含めない）

---

## 4. Metrics（固定）
### 4.1 必須メトリクスカテゴリ（固定）
- Uptime/heartbeat（componentごと）
- Throughput（ingest rate / persist rate / execution rate）
- Latency（E2E：intent→result、ingest→persist）
- Error rates（kind別：429/auth/protocol/integrity）
- Backpressure/backlog（storage/collector）
- Quarantine counts/durations（collector）
- Kill-Switch level（execution）
- Safety mode current + transitions（crosscut）
- Observability health itself（監視対象の欠損）

### 4.2 Missing targets（固定）
- 監視対象（targets）の欠損は “UNKNOWN” として扱う
- UNKNOWN は gate_results に反映され、crosscut safety で SAFE へ寄る

---

## 5. Traces（固定）
### 5.1 分散トレースの固定要件
- すべてのコンポーネントで同一 trace_id を伝播できること（可能な範囲で）
- 実装が無い場合も “無いこと” を明示できる（capabilitiesで宣言）

---

## 6. SLI/SLO（固定：枠組み）
### 6.1 SLI（固定枠）
SLIは最低限以下の型に分類される：
- Availability（稼働）
- Freshness（新鮮性）
- Completeness（欠損の少なさ）
- Correctness（整合性）
- Latency（遅延）
- Safety compliance（安全遵守：SAFE時に実行していない等）

### 6.2 SLO（固定枠）
- SLOの目標値は Policy（本書では固定しない）
- ただし SLO違反は必ず “原因と証拠参照” を持って報告できる（audit/integrity/replay）

---

## 7. Alerting（固定：分類と必須情報）
### 7.1 Alert クラス（固定）
- P0: Safety / Integrity catastrophe（例：EMERGENCY_STOP、Integrity FAIL）
- P1: Data quality severe（gap増、quarantine増、persist滞留）
- P2: Degradation（遅延増、429増、部分欠損）
- P3: Informational（再接続、回復、デプロイ）

### 7.2 Alert payload（固定必須項目）
アラートには最低限これを含む：
- severity（P0-P3）
- component
- window（開始/現在）
- symptom（何が起きた）
- suspected causes（候補）
- evidence refs（gate_results_ref / integrity_report_ref / audit_event ids / replay pointers）
- recommended next action（runbook link）

---

## 8. Diagnostics（固定）
### 8.1 Health endpoints（固定要求）
各主要コンポーネントは少なくとも以下を提供できる：
- `/healthz`：生存 + 依存関係（DB/queue等）の簡易状態（秘密なし）
- `/readyz`：トラフィックを受けてよいか（collector/executionで重要）
- `/metrics`：メトリクス

### 8.2 Diagnostics snapshot（固定）
Support Bundle 生成時に、最低限：
- /healthz 出力（redacted）
- /metrics スナップショット（redacted）
を含められる（crosscut bundle spec）

---

## 9. Support Bundle Triggers（固定枠組み）
トリガの条件値は Policy に置くが、枠組みは固定：

- Safety transition to SAFE / EMERGENCY_STOP
- Integrity report FAIL / UNKNOWN
- Gate results FAIL / UNKNOWN（observability欠損含む）
- Collector quarantine P0 threshold exceed
- Execution kill-switch set to BLOCK/FLATTEN (policy-defined)

固定ルール：
- bundle生成は audit_event に必ず残す（manifest ref）
- bundleは secret-free（crosscut）

---

## 10. Safety coupling（固定）
- Observability missing → gate UNKNOWN → Safety Mode at least SAFE
- “監視が死んでいるのに実行だけ動く” を許さない
- この連動は policy 値で “緩和” できない（意味の変更は禁止）

---

## 11. テスト/検証観点（DoD）
最低限これが検証できること：

1) targets 欠損で gate が UNKNOWN になり SAFE へ寄る
2) SAFE/EMERGENCY_STOP 遷移で audit_event が残る
3) 禁止キー混入がログ/バンドルに出ない（scanで防止）
4) collector/quarantine/backpressure がメトリクスで可視化される
5) アラートが evidence refs と runbook link を含む
6) bundle生成が audit_event から辿れる

---

## 12. Policy/Runbookへ逃がす点
- SLO目標値、アラート閾値、通知抑制、bundle生成条件、保持期間
- 復旧手順とオンコール運用
→ Policy/Runbookへ（意味は変えない）

---
End of document
