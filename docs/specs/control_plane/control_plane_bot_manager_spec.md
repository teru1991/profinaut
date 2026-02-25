# Control Plane / Bot Manager Core Spec v1.0（固定仕様）
Bot lifecycle / Operator actions / Dangerous Ops boundary / Safety coupling

- Document ID: CP-BOT-MANAGER-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): G（Control Plane / Dashboard / Bot Manager）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - 認可マトリクス詳細、時間窓、通知、UI文言 → `docs/policy/**`
  - 運用手順/エスカレーション → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
Control Plane は “人間が触れる境界” であり、事故はここから起きる。
本仕様は **誤操作・誤爆・権限逸脱・監査欠落** を設計で封じ、ボット運用を安全にする。

必達要件（固定）：
1) **Every operator action is explicit**：暗黙動作禁止（特に live）
2) **Dangerous ops are challenge-confirmed**：危険操作は必ず challenge/confirm
3) **Safety coupling**：SAFE/EMERGENCY_STOP では実行系操作を許さない（例外は固定仕様で禁止）
4) **Audit-first**：操作は必ず audit_event に残る（拒否も残る）
5) **Least privilege UI**：権限が無い操作はUIでも見せない/実行できない
6) **Idempotent commands**：同じコマンドが二重送信されても二重効果を起こさない
7) **Stable bot lifecycle**：状態機械が揺れず、復旧・停止が確実に行える

---

## 1. Control Plane の責務境界
### 1.1 In（対象）
- Bot/Job のライフサイクル管理（start/stop/pause/resume）
- 戦略設定の適用（config apply の境界）
- mode 切替（paper/shadow/live）
- 危険操作の challenge/confirm UI/API（固定規約）
- 実行前のゲート参照（Safety/Gate/Kill-Switch）
- 監査イベントの発行（操作意図・確認・結果）
- Support Bundle 生成の起動（固定枠組み）

### 1.2 Out（対象外）
- 具体の戦略ロジック
- 具体の取引所API方言
- 具体の観測スタック設定

---

## 2. Bot Lifecycle（固定：状態機械）
### 2.1 Bot State（固定語彙）
- `CREATED`：作成済み（まだ動作していない）
- `IDLE`：停止（安全に停止済み）
- `RUNNING_PAPER`：paper実行中
- `RUNNING_SHADOW`：shadow実行中（実発注なし）
- `RUNNING_LIVE`：live実行中
- `PAUSED`：一時停止（再開可能）
- `STOPPING`：停止処理中（graceful shutdown）
- `ERROR`：異常停止（原因あり）
- `QUARANTINED`：隔離（安全上の理由で自動再開しない）

固定ルール：
- `RUNNING_LIVE` は “明示操作” なしに到達しない
- `ERROR`→`RUNNING_*` へは自動で戻らない（原因解除＋明示再開）

### 2.2 Transition rules（固定）
- `CREATED`→`IDLE`：初期化完了
- `IDLE/PAUSED/ERROR`→`RUNNING_PAPER/SHADOW`：許容（ただし gates pass）
- `IDLE/PAUSED/ERROR`→`RUNNING_LIVE`：dangerous-op + safety NORMAL + gates pass 必須
- `RUNNING_*`→`PAUSED`：許容（ただし audit）
- `RUNNING_*`→`STOPPING`→`IDLE`：graceful を基本
- `ANY`→`QUARANTINED`：安全上の隔離（安全状態/整合性/監視欠損 等）

---

## 3. Environment / Mode Isolation（固定）
- dev/stage/prod は完全分離
- paper/shadow/live は明示切替
- mode=live を “暗黙に継承” しない（UI/APIとも）

---

## 4. Dangerous Operations（固定）
### 4.1 Dangerous op の定義（固定例）
- live 実行開始/再開
- live の設定適用（戦略パラメータ変更含む）
- Kill-Switch の緩和/解除
- Gate の無効化/緩和
- 監視の無効化
- 秘密（secret_ref）関連の差し替え/ローテーション実行
- 破壊的データ操作の起動（削除/リストア/強制コンパクション）
- QUARANTINED の解除

### 4.2 Challenge/Confirm（固定）
crosscut safety の §6 に完全準拠し、Control Plane は次を実装境界として固定：
- challenge 発行（challenge_id, requires, expiry）
- confirm（actor, reason, within window）
- reject/expire の明確化

固定ルール：
- confirm のない dangerous op は実行しない
- confirm は “同じactorの誤爆” を防ぐために明示操作を要求（UI/CLI/ APIのいずれでも）

---

## 5. Safety / Gate / Kill-Switch との連携（固定）
### 5.1 Safety（固定）
- safety_mode が SAFE/EMERGENCY_STOP のとき：
  - live start/resume は禁止
  - config apply は禁止（原則）
  - 許容されるのは “証拠採取/停止/診断” 系のみ

### 5.2 Runtime gate（固定）
- start/resume/apply の前に gate_results を参照し、UNKNOWN/FAIL は拒否が基本
- 観測不能（UNKNOWN）は “健康” ではない（Observability honesty）

### 5.3 Execution Kill-Switch（固定）
- Control Plane は Kill-Switch を “設定する/強める” 方向は常に許可できる（権限があれば）
- “緩める” は dangerous op 扱い（challenge/confirm + audit）

---

## 6. Command Idempotency（固定）
すべての操作コマンドは idempotent であること。

必須フィールド（固定）：
- `command_id`（idempotency key）
- `actor`（identity）
- `target`（bot_id / group_id）
- `requested_transition`（例：IDLE→RUNNING_LIVE）
- `trace_id/run_id/schema_version`

固定ルール：
- 同じ command_id は二重効果を起こさない
- 二重送信は “抑止” し、audit に `duplicate_suppressed` を残せる

---

## 7. Audit（固定）
### 7.1 必須イベント（固定）
最低限、以下を audit_event として残す：
- `controlplane.command.requested`（意図）
- `dangerous_op.challenge` / `dangerous_op.confirm` / `dangerous_op.reject`
- `controlplane.command.applied`（結果）
- `controlplane.command.denied`（拒否：理由）
- `bot.state.transition`（状態遷移）
- `support_bundle.created`（bundle生成時）
- `safety.transition` / `execution.killswitch.set`（参照）

秘密値は含めない。

### 7.2 Audit不能＝実行しない（固定）
audit_event を出せない状態では、危険操作は実行禁止。
（証明不能＝安全ではない）

---

## 8. Support Bundle 起動（固定枠組み）
Control Plane は bundle を生成要求できる。
- 生成トリガ値はPolicyだが、以下を固定：
  - 生成は dangerous op になり得る（個人情報/環境情報が含まれるため）
  - 生成後は audit_event で manifest ref を残す
  - bundleは secret-free（crosscut準拠）

---

## 9. UI/UX（固定：要件のみ）
UI実装は自由だが、固定要求として：
- live操作は視覚的に明示（色/ラベル/二重確認）
- dangerous op は “手順を飛ばせない” UI
- 権限が無い操作は表示/実行できない
- 状態（Safety/Gate/Kill-Switch）が常に見える
- 失敗時は “次の手順（runbook link）” を提示できる

---

## 10. 失敗モードと縮退（固定）
- gate UNKNOWN（監視欠損） → live系操作拒否
- safety SAFE/EMERGENCY_STOP → 実行操作拒否、停止/診断のみ
- botが STOPPING のまま固着 → runbook誘導 + bundle生成
- QUARANTINED → 自動復帰しない（明示解除のみ）

---

## 11. テスト/検証観点（DoD）
最低限これが検証できること：

1) live start が challenge/confirm なしでは絶対に通らない
2) SAFE/EMERGENCY_STOP で live 操作が拒否される
3) command_id の二重送信で二重効果が起きない
4) 拒否・確認・遷移が audit_event に必ず残る
5) 権限無しユーザが危険操作を実行できない
6) QUARANTINED が自動解除されない

---

## 12. Policy/Runbookへ逃がす点
- 認可マトリクス詳細、challenge window、通知、UI文言
- 復旧/エスカレーション手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
