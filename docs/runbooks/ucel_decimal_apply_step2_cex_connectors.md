# Apply Step 2 (SSOT): CEX connectors — f64 purge + guarded ingestion

この手順は Step1（ucel-core / serde / OrderGate）完了後に実施する。

## 0) 前提
- Step1 が完了し、以下が使える:
  - ucel_core::Decimal
  - ucel_core::decimal::serde::{deserialize_decimal_execution|observation|balance}
- 適用する patch が docs/patches/ucel に存在する:
  - UCEL-DECIMAL-IMPL-CEX-003.*.patch（coinbase/upbit/binance-options/deribit/bitbank/binance-coinm/htx/sbivc）
  - UCEL-DECIMAL-REMAIN-3-001.cex-guard.patch（value-class serdeへ寄せる）

## 1) 適用順（この順番固定）
1) まず f64排除 + guarded ingestion（各CEXのpatch）:
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.coinbase.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.upbit.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-options.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.deribit.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.bitbank.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-coinm.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.htx.patch
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.sbivc.patch
2) 次に value-class serde への寄せ（代表例）:
   - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-3-001.cex-guard.patch

## 2) テスト（まずは個別で潰す）
- cargo test -p ucel-cex-coinbase
- cargo test -p ucel-cex-upbit
- cargo test -p ucel-cex-binance-options
- cargo test -p ucel-cex-deribit
- cargo test -p ucel-cex-bitbank
- cargo test -p ucel-cex-binance-coinm
- cargo test -p ucel-cex-htx
- cargo test -p ucel-cex-sbivc

## 3) 典型修正（迷いゼロ）
### 3.1 binance-options: parse_num 戻り型変更の伝播
現象:
- parse_num が Decimal を返すようになり、呼び出し側が f64 を期待して失敗
対処（固定）:
- 呼び出し側の型も Decimal に変更する（f64へ戻さない）
- JSON payload/ログ出力は Decimal::to_string() を使う

### 3.2 deribit/htx: wrapper 型（GDecimal）導入による変換
現象:
- bids/asks が (GDecimal, GDecimal) や [GDecimal;2] になって型不一致
対処（固定）:
- 価格/数量に代入する直前に .0 を取り出す
  - 例: price: v.0, qty: w.0
- もしくは From実装がある場合は into() を使う

### 3.3 upbit: Option<f64> -> Option<Decimal>
現象:
- Option型が変わって unwrap_or_default 等で不一致
対処:
- unwrap_or_default は Decimal ならそのまま可
- ただし serde default が効いているか確認（#[serde(default)]）

### 3.4 parse::<Decimal>() の無検証残存（bitbank/binance-coinm等）
方針:
- 少なくとも負値拒否
- 可能なら value-class serde を使う（文字列→JSON→deserialize が必要な場合もあるが、入口wireはserde withで解決する）

## 4) 監査（この時点でやる）
- f64残存（取引値のみ）:
  - rg -n ":\s*f64\b" ucel/crates/ucel-cex-*
- 無検証 parse:
  - rg -n "parse::<Decimal>\(\)" ucel/crates/ucel-cex-*
期待:
- 重大箇所（price/qty/amount/volume等）の f64 が消えている
- parse はガード済み関数へ寄っている

## 5) 完了条件（Step2）
- 上記8crateの cargo test が全て成功
- 監査で重大な f64 残存が無い
