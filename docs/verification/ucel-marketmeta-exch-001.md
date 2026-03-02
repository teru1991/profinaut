# UCEL-MARKETMETA-EXCH-001 Verification

## 変更ファイル一覧
- `ucel/crates/ucel-cex-binance/src/symbols.rs`
- `ucel/crates/ucel-cex-binance-usdm/src/symbols.rs`
- `ucel/crates/ucel-cex-binance-coinm/src/symbols.rs`
- `ucel/crates/ucel-cex-binance-options/src/symbols.rs`
- `ucel/crates/ucel-cex-bitmex/src/symbols.rs`
- `ucel/crates/ucel-cex-coinbase/src/symbols.rs`
- `ucel/crates/ucel-cex-deribit/src/symbols.rs`
- `docs/verification/ucel-marketmeta-exch-001.md`

## 実装確認
- 各 `symbols.rs` に `pub async fn fetch_symbol_snapshot() -> Result<ucel_symbol_core::Snapshot, String>` を追加。
- RESTのDTOを `StandardizedInstrument` へマップし、`Snapshot::new_rest(instruments)` を返す実装に統一。
- `tick/step` は必須化し、欠損時に `Err` を返す。
- `min_qty/min_notional` は `Option` のまま取り込み。
- 既存 `fetch_all_symbols` は additive に維持（破壊的変更なし）。

## テスト
- 各 `symbols.rs` に fixture JSON の parse→map unit test を追加。
- 各 `symbols.rs` に `tick/step` 欠損で失敗する回帰テストを追加。

実行コマンド:
- `cd ucel && cargo fmt`
- `cd ucel && cargo test -p ucel-cex-binance -p ucel-cex-binance-usdm -p ucel-cex-binance-coinm -p ucel-cex-binance-options -p ucel-cex-bitmex -p ucel-cex-coinbase -p ucel-cex-deribit`

## Secrets scan
実行コマンド:
- `cd /workspace/profinaut && rg -n "AKIA|ASIA|BEGIN (RSA|EC|OPENSSH|DSA) PRIVATE KEY|xoxb-|ghp_[A-Za-z0-9]{20,}" ucel/crates/ucel-cex-binance/src/symbols.rs ucel/crates/ucel-cex-binance-usdm/src/symbols.rs ucel/crates/ucel-cex-binance-coinm/src/symbols.rs ucel/crates/ucel-cex-binance-options/src/symbols.rs ucel/crates/ucel-cex-bitmex/src/symbols.rs ucel/crates/ucel-cex-coinbase/src/symbols.rs ucel/crates/ucel-cex-deribit/src/symbols.rs`

結果:
- 0 hit（今回変更ファイル中に機密文字列パターンなし）
