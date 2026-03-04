# B-STEP5 Verification

## Changed files
- git diff --name-only
- dashboard_api/models.py
- docs/policy/change_mgmt_policy.json
- docs/policy/llm_egress_policy.json
- docs/reports/access_review_latest.md
- docs/runbooks/security/rotation_and_revocation.md
- docs/specs/security/egress_and_governance.md
- docs/status/trace-index.json
- docs/verification/B-STEP5.md
- libs/safety_core/access_review.py
- libs/safety_core/authz.py
- libs/safety_core/change_mgmt.py
- libs/safety_core/egress_guard.py
- libs/safety_core/egress_policy.py
- libs/safety_core/secret_access.py
- scripts/access_review_report.py
- scripts/policy_tool.py
- tests/test_access_review_report.py
- tests/test_egress_guard.py
- tests/test_no_plaintext_secrets_in_prod.py

## What/Why
- egress（外部送信）をpolicy+scanで強制ブロックし、prodでenv/plaintext secretsを完全拒否、月次AccessReviewとChangeMgmtの成果物を追加してDomain Bを運用完成させた。
- `egress_guard` で redaction/scan 後の外部送信判定を fail-closed 化し、`llm` / `public_http` を既定拒否。
- `change_mgmt_policy` を読み込む `ChangeMgmtPolicy` を追加し、`Authz.is_dangerous()` が controlled_changes を dangerous ops 扱いするよう接続した。
- 秘密取得の棚卸し結果に基づき、`dashboard_api/models.py` の DB URL 取得を SecretRef 入口（`secret_access`）へ寄せた。

## Secret access inventory evidence (required)
- 事前 inventory（grep 証拠）:
  - `rg -n "(API_KEY|SECRET|TOKEN|PASSWORD|AUTHORIZATION|BEARER|FILEENC_PASSPHRASE)" -S libs dashboard_api`
  - `rg -n "(os\.environ|getenv\(|dotenv|\.env)" -S libs dashboard_api`
  - 抽出された秘密/環境参照のうち、アプリ入口側で該当した `dashboard_api/models.py:get_database_url`（`os.getenv("DATABASE_URL", ...)`）を置換対象とした。
- 置換証拠（diff要点）:
  - 変更前: `os.getenv("DATABASE_URL", "...")`
  - 変更後: `SECRETREF_DATABASE_URL` がある場合に `get_secret_from_env_ref("SECRETREF_DATABASE_URL")` を使用。
  - フォールバックはローカル開発用固定値のみで、prod運用では SecretRef 経由を強制可能。

## Self-check results
- Allowed-path check OK: (awk)
  - `git diff --name-only | awk '{ ok=($0 ~ /^docs\// || $0 ~ /^libs\// || $0 ~ /^dashboard_api\// || $0 ~ /^tests\// || $0 ~ /^scripts\// || $0 ~ /^\.github\/workflows\// || $0=="pyproject.toml" || $0=="requirements-dev.txt" || $0=="package.json"); if(!ok) print $0 }'`
  - 結果: 空（OK）
- Tests:
  - `pytest -q tests/test_no_plaintext_secrets_in_prod.py tests/test_egress_guard.py tests/test_access_review_report.py tests/test_danger_ops_gate.py tests/test_dual_run_lease_guard.py tests/test_fileenc_crypto_v1.py tests/test_secretref_and_provider.py tests/test_redaction_no_leak.py`
  - 結果: `21 passed, 1 warning`
  - `pytest -q`
  - 結果: 失敗（既存不具合）`ModuleNotFoundError: worker` と `dashboard_api/main.py` SyntaxError で収集失敗
- Build:
  - `python -m compileall .`
  - 結果: 失敗（既存不具合）`dashboard_api/main.py` の SyntaxError（未クローズ括弧）
- policy validate/hash:
  - `python scripts/policy_tool.py validate docs/policy/llm_egress_policy.json` => OK
  - `python scripts/policy_tool.py validate docs/policy/change_mgmt_policy.json` => OK
  - `python scripts/policy_tool.py hash docs/policy/llm_egress_policy.json` => SHA256 出力
- trace-index json.tool OK（更新した場合）:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => OK
- Secrets scan:
  - 実secret混入なし（grepで確認、tests/policyはダミー値のみ）
- docsリンクチェック:
  - `rg -n "docs/" docs/specs/security/egress_and_governance.md docs/runbooks/security/rotation_and_revocation.md`

## ★履歴確認の証拠
- `git log --oneline --decorate -n 50` / `git log --graph --oneline --decorate --all -n 80` / `git log --merges --oneline -n 30`
  - 直近 HEAD は `7af141f`（B-STEP1〜4をまとめた安全基盤）で、B-STEP5はその運用仕上げとして整合。
- `git show HEAD`
  - 前コミットの内容が redaction/secrets/fileenc/danger-ops/authz/audit-health を網羅しており、今回の egress/governance 完成に直接接続。
- `git merge-base HEAD origin/main`
  - この環境は `origin/main` が無く取得不可（remote未設定）。
- `git branch -vv` / `git reflog -n 30`
  - `feature/b-step5-001` は `work` 先頭から分岐し、段階実装の履歴と矛盾なし。
- blame/grep で分かった “秘密取得の散在” と局所置換理由:
  - grepで確認された dashboard 側の env 直接読み取りは `dashboard_api/models.py` の DB URL 取得。
  - 大規模置換を避け、最小差分で `secret_access` 経由に集約（コンフリクト最小化）。
- “不足があったため追加実装した” 根拠:
  - B契約では egress・change mgmt・access review の成果物が必須。既存実装にはこの3点が無かったため、policy + guard + report/CLI + tests を具体実装して補完。
