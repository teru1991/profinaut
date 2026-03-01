# Verification — UCEL-DECIMAL-A-CORE-001

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-A-CORE-001.md
- docs/patches/ucel/UCEL-DECIMAL-A-CORE-001.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-a-core-001.md

## 2) What/Why
- ucel-core の既存変更内容を、実コード反映前の「完全パッチ（コピペ用）」として docs に固定しました。
- 対象は `types.rs` / `lib.rs` で、価格/数量/残高の `f64` を `Decimal` に統一する変更を明示しています。
- 将来タスクで導入される decimal policy/newtypes との整合を崩さない注記をパッチ内に含めました。
- task追跡のため、trace-index に `UCEL-DECIMAL-A-CORE-001` のみ追加しました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
