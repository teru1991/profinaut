# Verification — UCEL-DECIMAL-IMPL-CEX-003

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.md
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.coinbase.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.upbit.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-options.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.deribit.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.bitbank.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-coinm.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.htx.patch
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.sbivc.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-impl-cex-003.md

## 2) What/Why
- CEXコネクタ群の f64 残存排除と guarded Decimal ingestion を実装反映可能なパッチ群として docs に固定しました。
- 8コネクタ分の適用パッチを分割し、反映工程での適用順序を明示して迷いを減らしています。
- Deribit/HTX の wrapper 型導入や Binance Options の Decimal 伝播など、コンパイル起点で追従すべき注意点を明文化しました。
- trace-index は `UCEL-DECIMAL-IMPL-CEX-003` のみ更新し、成果物と検証証跡を紐付けています。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
