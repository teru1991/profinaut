UCEL-DECIMAL-C2-CEX-001 Patch SSOT (CEX: bitbank/binance-coinm/htx/sbivc)

目的
	•	Decimalを使っていても、parse::<Decimal>() を無検証で通すと不正値が侵入する
	•	“central guard を必ず通す” ため、共通デシリアライズ/変換関数へ寄せる
	•	最終的には ucel-core の DecimalPolicy/serde guard を参照する

対象
	•	ucel/crates/ucel-cex-bitbank/src/lib.rs
	•	ucel/crates/ucel-cex-binance-coinm/src/lib.rs
	•	ucel/crates/ucel-cex-htx/src/lib.rs
	•	ucel/crates/ucel-cex-sbivc/src/lib.rs

適用（コード反映工程で実施）
	•	git apply docs/patches/ucel/UCEL-DECIMAL-C2-CEX-001.patch
	•	cargo test -p ucel-cex-bitbank -p ucel-cex-binance-coinm -p ucel-cex-htx -p ucel-cex-sbivc
