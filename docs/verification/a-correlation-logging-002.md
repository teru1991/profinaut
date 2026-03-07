# A-CORRELATION-LOGGING-002 Verification

## 1) Changed files (`git diff --name-only`)
- contracts/observability/contract_constants.py
- dashboard_api/main.py
- docs/contracts/README.md
- docs/contracts/observability/correlation.schema.json
- docs/contracts/observability/log_event.schema.json
- docs/specs/platform_foundation_spec.md
- docs/status/trace-index.json
- libs/observability/contracts.py
- libs/observability/core.py
- libs/observability/correlation.py
- libs/observability/http_contracts.py
- libs/observability/logging.py
- libs/observability/middleware.py
- services/dashboard-api/app/main.py
- services/execution/app/main.py
- services/marketdata/app/main.py
- tests/test_contracts.py
- tests/test_observability_contract_capabilities.py
- tests/test_observability_contract_correlation_headers.py
- tests/test_observability_contract_healthz.py
- tests/test_observability_log_contract_schema.py
- tests/test_observability_logging_required_keys.py
- tests/test_observability_middleware_injects_headers_and_logs.py
- tests/test_observability_process_run_id.py
- docs/verification/a-correlation-logging-002.md

## 2) What / Why
- A-2/A-9 の基盤として、correlation source-of-truth（request_id/trace_id/run_id/event_id/component/source/schema_version）を `libs/observability/correlation.py` に集約した。
- middleware で request.state / contextvar / response headers を統一注入し、`X-Request-ID` / `X-Trace-ID` / `X-Run-ID` / `X-Schema-Version` を標準化した。
- JSON logging envelope を strict required key で固定し、request start/finish/error を correlation 付きで発行するようにした。
- A-ERROR-ENVELOPE-001 の error context builder を correlation source-of-truth 参照へ揃え、error body と response headers の相関整合を担保した。
- execution / marketdata / dashboard-api（+ legacy dashboard_api）の bootstrap 変更は middleware 導入点に限定し、業務ロジック改変を避けた。

## 3) Self-check results
- Allowed-path check: OK
  - `git diff --name-only | awk ...` => empty
- Tests added/updated: OK
  - Added: `tests/test_observability_process_run_id.py`
  - Updated: `tests/test_observability_contract_correlation_headers.py`, `tests/test_observability_logging_required_keys.py`, `tests/test_observability_middleware_injects_headers_and_logs.py`, `tests/test_contracts.py`
- Build/Unit command results:
  - `python -m py_compile ...` => PASS
  - `pytest tests/test_observability_process_run_id.py -q` => PASS
  - `pytest tests/test_observability_contract_correlation_headers.py -q` => PASS
  - `pytest tests/test_observability_logging_required_keys.py -q` => PASS
  - `pytest tests/test_observability_middleware_injects_headers_and_logs.py -q` => PASS
  - `pytest tests/test_contracts.py -q` => PASS
  - `pytest tests/test_observability_contract_healthz.py tests/test_observability_contract_capabilities.py tests/test_observability_log_contract_schema.py -q` => FAIL（環境依存: `jsonschema` モジュール未導入）
- trace-index json.tool: OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- Secrets scan: OK（新規 correlation/log 出力に秘密値直出しなし）
  - `rg -n "(api_key|token|secret|password)" ...`
- docsリンク存在チェック: OK（今回追加/更新の docs 参照は `docs/` 配下のみ）
- schema validation: partial
  - `python -m json.tool docs/contracts/observability/correlation.schema.json > /dev/null` => PASS
  - `python -m json.tool docs/contracts/observability/log_event.schema.json > /dev/null` => PASS
  - `python scripts/validate_json_schemas.py` => FAIL（環境依存: `jsonschema` 未導入）

## 4) 履歴確認の証拠
- 実行コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show --stat --oneline e57736e`
  - `git show --stat --oneline 3fe199d`
  - `git show --stat --oneline 1be48a7`
  - `git blame -w docs/specs/platform_foundation_spec.md | head -n 40`
  - `git blame -w libs/observability/core.py | head -n 40`
  - `git blame -w libs/observability/correlation.py | head -n 40`
  - `git blame -w libs/observability/middleware.py | head -n 40`
  - `git blame -w libs/observability/logging.py | head -n 40`
  - `git blame -w services/execution/app/main.py | head -n 40`
  - `git blame -w services/marketdata/app/main.py | head -n 40`
  - `git blame -w services/dashboard-api/app/main.py | head -n 40`
  - `git blame -w dashboard_api/main.py | head -n 40`
  - `git reflog -n 30`
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git show --stat --oneline 809e1de`
- 要点:
  - observability correlation/logging は C-OBS 系（1be48a7/3fe199d）で先行導入済みだが、run_id が request毎に変わる実装が残っていたため process-stable 化が不足していた。
  - A-ERROR-ENVELOPE-001（e57736e）で error envelope は導入済みだが、response headers / error context / logs の厳密一致を middleware 起点で固定する必要があった。
  - merge 運用は merge commit 中心（PR merge + branch merge）。revert された logging format は確認範囲内で検出されず。
  - `services/dashboard-api/app/main.py` は継続的に更新される現役導線、`dashboard_api/main.py` は legacy だが `tests/test_api.py` import 導線が残るため互換目的で middleware を同等導入。
  - run_id の今回固定方針: process-stable を default、inbound override は trusted flag + format 妥当時のみ許可。
  - healthz/capabilities/metrics への適用は「exemptにしない」方針に合わせ middleware と contract helper 双方で headers を維持。
- リモート確認補足:
  - この環境では remote 未設定のため `origin/master` 比較系コマンドは `bad revision`。ローカル履歴・blame・reflogで代替確認。

## 5) Contract alignment
- `docs/contracts/observability/correlation.schema.json` required:
  - `schema_version`, `trace_id`, `run_id`, `component`, `source`, `op`, `emitted_at`
- `docs/contracts/observability/log_event.schema.json` required:
  - `schema_version`, `timestamp`, `level`, `message`, `component`, `trace_id`, `run_id`
- 一致確認:
  - middleware headers (`X-Request-ID`, `X-Trace-ID`, `X-Run-ID`, `X-Schema-Version`)
  - error context (`request_id`, `trace_id`, `run_id`, `component`)
  - logs (`request_started`/`request_finished`/`request_failed`) の IDs が同値になることを回帰テストで確認。

## 6) Compatibility notes
- 既存成功レスポンス body shape は変更していない（追加は headers / logs / context 主体）。
- strict logging は wrapper 層で導入し、既存 logging backend の全面置換は実施していない。
- dashboard legacy 入口は削除せず互換導線として維持。
