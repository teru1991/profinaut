# Runbook Index & Format Spec v1.0（固定）
Document ID: SYS-RUNBOOK-INDEX-FORMAT-SPEC
Status: Canonical / Fixed Contract
Scope: runbooks の “書式・索引・エラー導線” を固定する（Runbook本文は可変）

## 0. 目的（Non-negotiable）
Runbook は可変だが、バラバラな書式は障害対応を遅らせる。
本書は以下を固定する：
- runbook の必須章立て
- error_code（SYS-ERROR-CATALOG-SSOT）との接続
- runbook index（docs/runbooks/README.md）で必ず辿れる導線

---

## 1. 参照（正本）
- Runbooks index（可変の正本）: `docs/runbooks/README.md`
- Error catalog（固定）: `docs/specs/system/error_catalog_ssot.md`
- Safety（固定）: `docs/specs/crosscut/safety_interlock_spec.md`
- Audit/Replay（固定）: `docs/specs/crosscut/audit_replay_spec.md`
- Support Bundle（固定）: `docs/specs/crosscut/support_bundle_spec.md`
- Observability（固定）: `docs/specs/observability/observability_slo_diagnostics_spec.md`

---

## 2. Runbook のファイル規約（推奨）
### 2.1 命名（推奨）
- `docs/runbooks/<domain_or_area>_<topic>_playbook.md`
  - 例：`collector_recovery_playbook.md`
  - 例：`execution_failures_playbook.md`

### 2.2 冒頭メタ（推奨だが強く推奨）
- Title / Version（Runbookは可変なので v1.0 の更新は柔軟）
- Scope（何の障害に効くか）
- Related error codes（ERR-... のリスト）
- Evidence（必要な証拠：audit/integrity/gate/replay/bundle）

---

## 3. Runbook の必須章立て（固定：MUST）
Runbook 本文は必ずこの順序で書く（順序の入替は禁止）。

1) **Symptoms**（何が起きたか）
2) **Observations**（何を見て確認するか：metrics/logs/traces）
3) **Auto-recovery**（自動で何が起きる/起きない）
4) **Manual actions**（手動介入：安全な順序）
5) **Evidence**（audit/integrity/gate/replay/support bundle で証拠化）
6) **Rollback/Recovery**（戻す/復旧：影響範囲と確認）
7) **Postmortem**（再発防止：decision/plan/policy/spec更新導線）

---

## 4. Error Catalog との連動（固定：MUST）
- `docs/specs/system/error_catalog_ssot.md` に登録された `operator_action=RUNBOOK` の error_code は、
  **必ず** runbook を持つ。
- runbook は `docs/runbooks/README.md` から 1クリックで辿れること。
- runbook 側は “Related error codes” を本文冒頭に記載する（推奨だが実質必須）。

---

## 5. 安全と秘密（固定：MUST）
- runbook に secret を書かない（鍵/トークン/署名/URLクエリ等）
- 手動介入手順は必ず “安全側の順序” を先に書く（例：STOP→ISOLATE→DRAIN→RESTORE）
- live 操作が絡む場合は、必ず environment/mode と safety_state を確認する導線を置く
  - `docs/specs/system/environment_mode_matrix.md`
  - `docs/contracts/safety_state.schema.json`

---

## 6. Runbook Index（docs/runbooks/README.md）の固定要件（MUST）
index は可変だが、最低限この情報を持つ：
- どのカテゴリに属するか（collector / execution / storage / onchain / security / release / bundle 等）
- 参照すべき crosscut/spec を示す（Safety/Audit/Bundle/Observability）
- 重大障害の入口が “迷わず” 分かる（Unknown/Degraded の扱い）

---

## 7. DoD（runbookが運用で機能する条件）
1) error_code→runbook→evidence の導線がある
2) 手動介入前に safety/environment/mode の確認が組み込まれている
3) 証拠（bundle/監査/整合）を残す手順がある
4) 事後に spec/policy/plan/decision へ反映する導線がある

---
End of document
