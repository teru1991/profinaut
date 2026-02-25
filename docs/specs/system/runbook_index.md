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
- docs/runbooks/<domain_or_area>_<topic>_playbook.md

### 2.2 冒頭メタ（推奨）
- Title / Version
- Scope
- Related error codes（ERR-...）
- Evidence（audit/integrity/gate/replay/bundle）

---

## 3. Runbook の必須章立て（固定：MUST）
1) Symptoms
2) Observations
3) Auto-recovery
4) Manual actions
5) Evidence
6) Rollback/Recovery
7) Postmortem

---

## 4. Error Catalog との連動（固定：MUST）
- operator_action=RUNBOOK の error_code は必ず runbook を持つ
- runbook は docs/runbooks/README.md から 1クリックで辿れること
- runbook 側は Related error codes を本文冒頭に記載する

---

## 5. 安全と秘密（固定：MUST）
- runbook に secret を書かない
- 手動介入手順は必ず安全側の順序を先に書く
- live 操作が絡む場合は environment/mode と safety_state を確認する導線を置く

---

## 6. Runbook Index（docs/runbooks/README.md）の固定要件（MUST）
index は最低限この情報を持つ：
- 障害カテゴリ
- 参照すべき crosscut/spec
- 重大障害の入口が迷わず分かる

---

## 7. DoD
1) error_code→runbook→evidence の導線がある
2) safety/environment/mode の確認が組み込まれている
3) 証拠（bundle/監査/整合）を残す手順がある
4) 事後に spec/policy/plan/decision へ反映する導線がある

---
End of document
