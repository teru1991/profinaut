UCEL-DECIMAL-C1-CEX-001 Patch SSOT (CEX: coinbase/deribit/upbit/binance-options)

目的
	•	WS/REST入力に f64 が残ると、丸め/tick/step以前に事故る（精度欠落）。
	•	Decimal化 + ガード適用（少なくとも負値や異常値を拒否）。
	•	最終的には ucel-core の Decimal policy/serde guard に統合する。

対象
	•	ucel/crates/ucel-cex-coinbase/src/lib.rs
	•	ucel/crates/ucel-cex-deribit/src/lib.rs
	•	ucel/crates/ucel-cex-upbit/src/lib.rs
	•	ucel/crates/ucel-cex-binance-options/src/lib.rs

適用（コード反映工程で実施）
	•	git apply docs/patches/ucel/UCEL-DECIMAL-C1-CEX-001.patch
	•	cargo test -p ucel-cex-coinbase -p ucel-cex-deribit -p ucel-cex-upbit -p ucel-cex-binance-options
