# UX / Human Factors / Safety UI Patterns Core Spec v1.0（固定仕様）
Safe UX patterns / Human error prevention / Dangerous ops flows / Truthful status display

- Document ID: UX-SAFETY-PATTERNS-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): X（UX / Human Factors）
- Depends-on（Fixed）:
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Control Plane: `docs/specs/control_plane/control_plane_bot_manager_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Reporting truth: `docs/specs/reporting/reporting_dashboard_explainability_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
- Policy separation（固定しない）:
  - 文言、色、タイムアウト値、通知の細部 → `docs/policy/**`
  - 運用手順/対応フロー → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
システム事故の多くは “UI/人間の誤操作” から起きる。
本仕様は、UXを安全装置として扱い、誤爆・誤認・見落とし・安易な緩和を設計で封じる不変条件を固定する。

必達要件（固定）：
1) **No silent mode**：live/paper/shadow の状態を曖昧にしない（常時表示）
2) **Dangerous ops are hard**：危険操作は“手順を飛ばせない”導線（challenge/confirm）
3) **Truthful status**：UNKNOWN/欠損/監視欠損を健康に見せない
4) **Least privilege UX**：権限が無い操作は見えない/押せない（403常態化を避ける）
5) **Explainability**：拒否/停止/隔離は「理由コード＋証拠リンク＋次の手順」を出せる
6) **Idempotent interactions**：二重クリック/再送で二重効果が起きない
7) **Auditability**：重要操作は監査される（拒否も含む）

---

## 1. 範囲（in / out）
### 1.1 In
- 安全UIパターン（危険操作、live切替、緩和操作）
- 状態表示の真実性（Safety/Gate/Integrity/Quality）
- 権限に基づくUI制御
- 事故防止（確認、タイムロック、明示入力）
- 説明可能性（理由と証拠）
- 監査に残すべきUXイベントの枠組み

### 1.2 Out
- 特定UIフレームワーク
- ピクセルレベルのデザイン
- 文言/色の最終決定（Policy）

---

## 2. Always-visible status (固定)
UIの主要画面では、常に以下を表示できること（固定要件）：

- System Safety Mode（NORMAL/SAFE/EMERGENCY_STOP）
- Execution Kill-Switch（ALLOW/CLOSE_ONLY/FLATTEN/BLOCK）
- Gate status（PASS/WARN/FAIL/UNKNOWN）
- Integrity status（PASS/WARN/FAIL/UNKNOWN）
- Environment（dev/stage/prod）
- Mode（paper/shadow/live）
- Freshness（表示データの時刻、stale判定）

固定ルール：
- UNKNOWN は緑で表示しない
- SAFE/EMERGENCY_STOP を目立たなくしない
- stale を最新として見せない

---

## 3. Mode switching patterns (固定)
### 3.1 paper/shadow/live 切替（固定）
- 切替は “明示操作” のみ（暗黙継承禁止）
- live への遷移は dangerous op（challenge/confirm必須）
- 切替UIは現在値と変更後値を並べて表示する（before/after）

### 3.2 Live誤爆防止（固定）
live開始（または再開）を押下できるのは以下が揃う場合のみ：
- safety_mode=NORMAL
- gate != FAIL/UNKNOWN（PolicyでWARN扱いは調整可）
- actor が十分な権限（Identity/Access）
- challenge/confirm を完了できる（UIで誘導）

---

## 4. Dangerous Operations UX (固定)
dangerous op は crosscut safety の challenge/confirm に完全準拠し、UXで“抜け道”を作らない。

### 4.1 必須UXステップ（固定）
- Step A: 意図の明示（何をするか、影響範囲、リスク）
- Step B: チャレンジ提示（challenge_id、期限、要求事項）
- Step C: 確認入力（理由、必要なら「対象IDの手入力」等）
- Step D: 最終確認（実行ボタン、取り消し可能性）
- Step E: 結果表示（成功/失敗、証拠リンク、次の手順）

固定ルール：
- confirm は “自動” にできない
- 期限切れは明確に拒否し、再チャレンジへ誘導
- 失敗時に “半端な状態” を隠さない（reason codes表示）

### 4.2 代表dangerous ops（固定例）
- live start/resume
- kill-switch緩和
- gate無効化
- quarantine解除
- 破壊的データ操作（削除/リストア）
- secrets rotate/revoke 実行
- 監視無効化

---

## 5. Least privilege UX (固定)
### 5.1 表示/操作の分離（固定）
- 閲覧だけのユーザは危険操作ボタンが表示されない
- operator は停止/診断など限定操作
- admin は challenge 発行可能
- break-glass は期限付きで明示表示（“今は特権状態”を強調）

### 5.2 拒否のUX（固定）
- 権限不足で押せない（disabled）+ 理由表示
- API拒否（403等）が起きた場合も、画面に「理由＋次の手順」を表示

---

## 6. Explainability patterns (固定)
拒否/停止/隔離/縮退は、必ず以下を提示できる：
- 結論（拒否/停止/隔離）
- 理由コード（例：KILLSWITCH_BLOCK / SAFETY_SAFE_MODE / GATE_UNKNOWN / INTEGRITY_FAIL）
- 証拠リンク（audit_event / integrity_report / gate_results）
- 推奨次アクション（runbookリンク）

固定ルール：
- “不明” の場合は UNKNOWN と明示し、断定しない

---

## 7. Idempotent interaction patterns (固定)
- 重要ボタンは二重クリック防止（in-flight表示）
- すべての操作は command_id を持ち、二重送信で二重効果を起こさない
- 再読み込みしても結果が “確定” として一致する（audit参照）

---

## 8. Error display & safety (固定)
- エラーは Standard Error Model（kind/code）で表示できる
- secret/PIIを表示しない（redaction済み）
- “再試行” は、再試行してよい場合のみ表示（retryable=true）

---

## 9. Audit hooks（固定）
UIは最低限以下のイベントを audit_event として残せる（秘密なし）：
- `ui.dangerous_op.challenge.viewed`
- `ui.dangerous_op.confirm.submitted`
- `ui.command.submitted`（command_id）
- `ui.command.denied`（reason codes）
- `ui.mode.switch.requested`
- `ui.export.requested`（該当する場合）

※ 監査対象範囲は Policy で調整できるが、“危険操作は必ず監査” は固定。

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) live切替が暗黙に起きず、dangerous op 導線が必須
2) UNKNOWN/欠損が健康表示されない
3) 権限無しユーザが危険操作を見えない/押せない
4) 拒否が理由コード＋証拠リンク＋次手順を表示する
5) 二重クリックで二重効果が起きない（command_id）
6) 重要操作が監査に残る

---

## 11. Policy/Runbookへ逃がす点
- 文言、色、タイムアウト、通知の細部
- 対応フロー（オンコール/復旧）
→ Policy/Runbookへ（意味は変えない）

---
End of document
