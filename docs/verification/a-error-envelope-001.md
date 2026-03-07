# A-ERROR-ENVELOPE-001 Verification

## 1) Changed files (`git diff --name-only`)
- dashboard_api/main.py
- docs/contracts/README.md
- docs/specs/platform_foundation_spec.md
- docs/specs/system/error_catalog_ssot.md
- docs/status/trace-index.json
- libs/observability/contracts.py
- libs/observability/core.py
- services/dashboard-api/app/main.py
- services/execution/app/main.py
- services/marketdata/app/main.py
- tests/test_contracts.py
- tests/test_standard_error_envelope.py
- contracts/schemas/common/error_envelope.schema.json
- docs/verification/a-error-envelope-001.md

## 2) What / Why
- A-1 の標準 error envelope を canonical shape として docs/specs に固定し、サービス実装と契約テストを同時に導入した。
- `libs/observability/core.py` に exception 分類・context 生成・handler 一括導入 helper を集約し、各 service main では登録呼び出しの局所変更に留めた。
- execution / marketdata / dashboard-api の例外応答を共通 envelope へ統一した。
- marketdata は degraded payload の top-level key を維持しつつ `error` subobject を標準 shape に更新した。
- contract 側に JSON Schema と fail-fast テストを追加し、必須項目欠落を確実に検知できるようにした。

## 3) Self-check results
- Allowed-path check: OK
  - Command: `git diff --name-only | awk ...` (no output)
- Tests added/updated: OK
  - `tests/test_standard_error_envelope.py` (new)
  - `tests/test_contracts.py` (updated)
- Build/Unit test command results:
  - `python -m py_compile libs/observability/core.py libs/observability/contracts.py services/execution/app/main.py services/marketdata/app/main.py services/dashboard-api/app/main.py dashboard_api/main.py tests/test_standard_error_envelope.py tests/test_contracts.py` => PASS
  - `pytest tests/test_standard_error_envelope.py -q` => PASS (4 passed)
  - `pytest tests/test_contracts.py -q` => PASS (12 passed)
  - `pytest tests/test_api.py tests/test_observability_metrics_required.py -q` => FAIL（既存環境/既存導線由来。`services.execution.app.exchange_gateway` import 経路、marketdata `/metrics` の既存 NameError、`dashboard_api` 側 `/metrics` 404）
  - `pytest tests -k "error_envelope or contracts or degraded" -q` => FAIL（既存環境依存: `jsonschema` 未導入、`worker` import 不足）
- trace-index json.tool: OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- Secrets scan: OK（追加した error message/details で秘密値の直出しなし）
  - `rg -n "(api_key|token|secret|password)" ...`
- docsリンク存在チェック: OK（今回追加 docs は repo 内相対リンクのみ）
- schema validation: OK
  - `python -m json.tool contracts/schemas/common/error_envelope.schema.json > /dev/null`

## 4) 履歴確認の証拠
- 実行した履歴確認コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show --stat --oneline 809e1de`
  - `git show --stat --oneline ab50966`
  - `git show --stat --oneline b6cd226`
  - `git show --stat --oneline 3343d5d`
  - `git show --stat --oneline d9ef765`
  - `git show --stat --oneline 2440cb8`
  - `git blame -w docs/specs/platform_foundation_spec.md | head -n 40`
  - `git blame -w libs/observability/core.py | head -n 40`
  - `git blame -w services/execution/app/main.py | head -n 40`
  - `git blame -w services/marketdata/app/main.py | head -n 40`
  - `git blame -w services/dashboard-api/app/main.py | head -n 40`
  - `git blame -w dashboard_api/main.py | head -n 40`
  - `git reflog -n 30`
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
- 要点:
  - 直近運用は merge commit 中心（PR merge + branch merge 混在）で、squash 専用ではない。
  - `libs/observability/core.py` は C-OBS-002 で observability 共通化済みであり、今回の error envelope 集約先として整合。
  - `services/*/app/main.py` の error handler は過去に個別実装されており、統一は未完了だった。
  - `services/dashboard-api/app/main.py` は直近更新が新しく（3343d5d）、dashboard 本線と判断。`dashboard_api/main.py` は legacy だが `tests/test_api.py` で import されるため互換対象として handler 導入。
  - marketdata の degraded payload は既存で `symbol/stale/degraded_reason/error` を持ち、top-level 互換維持前提があるため error subobject 内のみ標準化。
  - `tests/test_contracts.py` は従来 pydantic model テストのみで schema fail-fast が不足していたため、error envelope contract テストを追加。
- リモート確認補足:
  - `origin/master` はこの環境に未設定で、`git merge-base HEAD origin/master` 等は実行不能（bad revision）。ローカル履歴と blame/reflog で代替確認。

## 5) Compatibility notes
- 成功系レスポンス shape は変更していない。
- 例外応答は canonical `{"error": {...}}` へ統一した。
- marketdata degraded payload は top-level key を維持し、`error` の内部を標準化した。
- 新規 schema と契約テストにより、`code/reason_code/kind/retryable/context.component` 欠落時に fail-fast する。

## 6) Open follow-up
- A-CORRELATION-LOGGING-002 で request correlation middleware を強化した際に、`context.trace_id/run_id` の取得網羅をさらに厳密化可能。
