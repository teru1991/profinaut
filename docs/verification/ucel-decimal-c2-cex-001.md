# Verification — UCEL-DECIMAL-C2-CEX-001

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-C2-CEX-001.md
- docs/patches/ucel/UCEL-DECIMAL-C2-CEX-001.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-c2-cex-001.md

## 2) What/Why
- Bitbank/Binance-CoinM/HTX/SBIVC 向けに、Decimal利用時の無検証 parse/serde を central guard 経由へ寄せる最小安全パッチを docs に固定しました。
- `parse::<Decimal>()` の直接利用をガード関数経由に置き換える方針と、serde deserialize時の負値拒否を明示しています。
- この変更は最終的に ucel-core の DecimalPolicy/serde guard SSOT へ統合する前段として位置付けています。
- trace-index は `UCEL-DECIMAL-C2-CEX-001` のみ更新し、成果物と検証証跡を紐付けました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
