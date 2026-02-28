UCEL-DECIMAL-B-SYMBOL-001 (ucel-symbol-core) Patch SSOT

目的
	•	ucel-symbol-core の round_price/round_qty/cmp_decimal は部分実装で、tick/stepやguardと乖離しやすい。
	•	よって ucel-core のSSOT（Decimal policy）へ委譲する。

対象
	•	ucel/crates/ucel-symbol-core/Cargo.toml
	•	ucel/crates/ucel-symbol-core/src/lib.rs

適用（コード反映工程で実施）
	•	git apply docs/patches/ucel/UCEL-DECIMAL-B-SYMBOL-001.patch
	•	cargo test -p ucel-symbol-core
