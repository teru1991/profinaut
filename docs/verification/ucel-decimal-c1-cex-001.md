# Verification — UCEL-DECIMAL-C1-CEX-001

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-C1-CEX-001.md
- docs/patches/ucel/UCEL-DECIMAL-C1-CEX-001.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-c1-cex-001.md

## 2) What/Why
- Coinbase/Deribit/Upbit/Binance-Options 向けに、f64 残存と無検証parse/serdeを縮退させる最小安全パッチを docs に固定しました。
- この段階では代表的なWS/主要wireの数値型を Decimal に寄せ、負値拒否の一時ガードを導入する内容を明示しています。
- 最終統合先は ucel-core の Decimal policy/serde guard SSOT であることをパッチ注記に固定しました。
- trace-index には `UCEL-DECIMAL-C1-CEX-001` のみ追加し、成果物と verification を紐付けています。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
