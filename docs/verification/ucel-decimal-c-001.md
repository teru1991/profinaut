# UCEL-DECIMAL-C-001 Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/decimal_policy/C_tests_scaffold.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-c-001.md

## 2) What / Why
- Decimal Policy の事故防止根拠として、3 本の新規テスト雛形（rounding / tick-step quantize / guard rejects invalid）を SSOT として docs に固定しました。
- 実装タスクでそのまま転記できるよう、各テストファイルを全コードで文書化しました。
- `trace-index` に TASK-ID `UCEL-DECIMAL-C-001` の branch/artifacts/verification_evidence を追加して追跡可能性を確保しました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照だけ）OK
- MISSING: none
