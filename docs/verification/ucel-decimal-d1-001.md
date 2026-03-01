# Verification — UCEL-DECIMAL-D1-001

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/decimal_policy_spec.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-d1-001.md

## 2) What/Why
- UCELのDecimal運用をSSOTとして固定する仕様書を新規作成しました。
- 仕様書には、丸め/比較/tick-step/拒否条件/例外/フラグ/責務境界/必須性の根拠を明文化しています。
- トレーサビリティのため、trace-indexにTASK-ID `UCEL-DECIMAL-D1-001` のエントリを追加しました。
- 自己検査結果を本ファイルに集約し、レビュー時の確認コストを下げています。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
