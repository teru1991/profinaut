# B-STEP4 Verification

## Changed files
- git diff --name-only
- dashboard_api/app.py
- dashboard_api/security_mw.py
- docs/policy/danger_ops_policy.json
- docs/policy/dual_run_policy.json
- docs/specs/security/danger_ops_and_authz.md
- docs/status/trace-index.json
- docs/verification/B-STEP4.md
- libs/safety_core/audit.py
- libs/safety_core/audit_health.py
- libs/safety_core/authz.py
- libs/safety_core/danger_ops.py
- libs/safety_core/lease_guard.py
- libs/safety_core/session.py
- tests/test_danger_ops_gate.py
- tests/test_dual_run_lease_guard.py

## What/Why
- deny-by-default 認可、dangerous ops gate（challenge/confirm+step-up）、audit health down→特権拒否、dual-run lease guard を実装し、テストでfail-closedを証明した。
- `audit.py` に監査ヘルス連動フックを最小追記し、書き込み成功/失敗を `AuditHealth` に反映できるようにした。
- dashboard API 側には既存ルートを書き換えずに統合点を追加するため、`dashboard_api/security_mw.py` と `dashboard_api/app.py` を新設した。

## Self-check results
- Allowed-path check OK: (awk)
  - `git diff --name-only | awk '{ ok=($0 ~ /^docs\// || $0 ~ /^libs\// || $0 ~ /^dashboard_api\// || $0 ~ /^tests\// || $0 ~ /^scripts\// || $0 ~ /^\.github\/workflows\// || $0=="pyproject.toml" || $0=="requirements-dev.txt" || $0=="package.json"); if(!ok) print $0 }'`
  - 結果: 空（OK）
- Tests:
  - `pytest -q tests/test_danger_ops_gate.py tests/test_dual_run_lease_guard.py`
  - 結果: `3 passed, 1 warning`
  - `pytest -q`
  - 結果: 失敗（既存不具合）`ModuleNotFoundError: worker` と `dashboard_api/main.py` SyntaxError で収集失敗
- Build:
  - `python -m compileall .`
  - 結果: 失敗（既存不具合）`dashboard_api/main.py` の SyntaxError（未クローズ括弧）
- trace-index json.tool OK（更新した場合）:
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
  - 結果: OK
- Secrets scan:
  - `rg -n "BEGIN PRIVATE KEY|ghp_|xox[baprs]-|AKIA" docs libs tests dashboard_api scripts`
  - 結果: policyに実データ/秘密なし（例のみ）
- docsリンクチェック: 今回触った docs 内の参照を確認
  - `rg -n "docs/" docs/specs/security/danger_ops_and_authz.md`

## ★履歴確認の証拠
- git log/merges/merge-base の要点（SHA/結論）
  - 直近履歴は `8cc664d`（B-STEP3）→ `51b3ff9`（B-STEP2）→ `63b61ab`（B-STEP1）で Domain B の段階導入と矛盾なし。
  - `git merge-base HEAD origin/main` はこの環境で `origin/main` 未設定のため取得不可（remote 未設定）。
- blameで分かった既存app入口の制約（なぜ最小差分にしたか）
  - `dashboard_api/app.py` は存在せず、`dashboard_api/routes/*.py` も存在しないため、統合点は新規追加で最小化した。
  - `dashboard_api/main.py` は既存のSyntaxErrorを含むため、目的外修正を避けてB-STEP4の統合点ファイルを独立追加した。
- 追加実装の根拠（B契約のdanger ops/audit down要件）
  - `audit health down -> dangerous ops deny` は `confirm()` 内 `audit_health.require_ok_for_danger_ops()` で強制。
  - step-up未達・トークン不正・期限切れ・スコープ/セッション不一致は全て fail-closed で拒否。
  - dual-run split brain は `LeaseGuard.acquire()` で競合時に必ず拒否。
