# Verification — UCEL-DECIMAL-B-SYMBOL-001

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-B-SYMBOL-001.md
- docs/patches/ucel/UCEL-DECIMAL-B-SYMBOL-001.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-b-symbol-001.md

## 2) What/Why
- ucel-symbol-core の独自丸め実装を段階的に縮退し、ucel-core 側SSOTへ寄せるためのパッチを docs に固定しました。
- 対象は `Cargo.toml` と `src/lib.rs` で、`rust_decimal` 直接依存から `ucel-core` 依存へ移行する内容を示しています。
- `round_*` は最終完全委譲前の中間段として、委譲方針の注記付き変更をパッチ化しています。
- trace-index には `UCEL-DECIMAL-B-SYMBOL-001` エントリのみ追加し、成果物と検証証跡を紐付けました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
