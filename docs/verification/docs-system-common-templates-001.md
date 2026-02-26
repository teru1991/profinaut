# Verification — DOCS-SYSTEM-COMMON-TEMPLATES-001

## 1) 変更ファイル一覧（`git diff --name-only`）
- docs/README.md
- docs/verification/docs-system-common-templates-001.md

## 2) 何をしたか
- Canonical 5ファイル（`docs/specs/system/*.md`）の存在を確認した。
- `docs/README.md` に「System Templates & Ledgers」セクションがあり、5ファイルへの導線があることを確認し、見出しを要件どおりに調整した。
- `docs/runbooks/README.md` に `docs/specs/system/runbook_index.md` へのリンクがあることを確認した。
- `docs/status/trace-index.json` の `tasks["DOCS-SYSTEM-COMMON-TEMPLATES-001"]` が必要artifact/verification_evidenceを含むことを確認した。
- 変更は docs/ 配下のみであることを検証した。

## 3) Link existence check（対象7ファイル内の `docs/` 参照）
MISSING: none

## 4) trace-index JSON 整形チェック
- `python -m json.tool docs/status/trace-index.json > /dev/null` : OK
