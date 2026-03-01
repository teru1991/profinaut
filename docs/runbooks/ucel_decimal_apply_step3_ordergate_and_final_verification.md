# Apply Step 3 (SSOT): OrderGate enforcement + final DoD verification

このステップが “tick/step適用をUCELが責務化” の完了点。

## 0) 前提
- Step1/Step2 が完了している（ucel-core/ucel-symbol-core/CEXのDecimal化が通っている）
- 以下 patch が存在する:
  - UCEL-DECIMAL-REMAIN-3-001.order-gate-kraken.patch
  - UCEL-DECIMAL-REMAIN-3-001.tests.patch

## 1) patch適用
1) 代表発注経路（Kraken）へ OrderGate 強制:
   - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-3-001.order-gate-kraken.patch
2) テスト追加（ucel-core）:
   - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-3-001.tests.patch

## 2) “他の発注経路” へ水平展開（固定手順）
目的: Kraken だけでは不十分。少なくとも “発注を持つCEX” 全てで同じパターンにする。

検索:
- rg -n "(add_order|create_order|place_order|submit_order|new_order)" ucel/crates/ucel-cex-*

修正方針（固定）:
- 発注payload生成直前で OrderGate を呼ぶ
- tick_size/step_size は symbol filters/catalog から取得して注入する（固定値は残さない）
- OrderGate の recommended_modes を使う（BUY=Ceil, SELL=Floor, Qty=ToZero）
- payloadの price/qty は string として送る（Decimal.to_string）

チェック:
- rg -n "OrderGate" ucel/crates/ucel-cex-*
期待: 発注経路があるcrateには OrderGate 呼び出しが存在

## 3) テスト
- cargo test -p ucel-core
- （krakenがある場合）cargo test -p ucel-cex-kraken
- 可能なら全CEX:
  - cargo test -p ucel-cex-coinbase -p ucel-cex-upbit -p ucel-cex-binance-options -p ucel-cex-deribit -p ucel-cex-bitbank -p ucel-cex-binance-coinm -p ucel-cex-htx -p ucel-cex-sbivc

## 4) 最終監査（DoD判定）
- f64残存:
  - rg -n ":\s*f64\b" ucel/crates | rg -n "(price|qty|amount|volume|bid|ask|free|locked|balance|mark|index|last|size)"
- 無検証 parse:
  - rg -n "parse::<Decimal>\(\)" ucel/crates
- OrderGate:
  - rg -n "OrderGate" ucel/crates/ucel-cex-*
- value-class serde:
  - rg -n "deserialize_decimal_(execution|observation|balance)" ucel/crates/ucel-cex-*

## 5) 完了条件（Step3 = 完全実装）
- [ ] 発注経路があるCEXは OrderGate を必ず通す
- [ ] tick/step違反は発注前に Err で止まる
- [ ] f64重大残存が無い
- [ ] parse/serdeで不正値侵入ができない（最低：負値、scale、max_abs、executionの0禁止）
- [ ] cargo test が全て通る
