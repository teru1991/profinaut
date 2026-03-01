UCEL-DECIMAL-IMPL-CEX-003 — CEX connectors: f64 purge + guarded Decimal ingestion (SSOT patches)

目的（事故防止）
	•	価格/数量を f64 で保持すると、tick/step適用以前に精度欠落が起き、発注/計算事故の原因になる。
	•	Decimal を使っていても、parse::<Decimal>() や serde で無検証に通すと 不正値（負値・scale超過・0禁止など）が侵入する。
	•	したがって各コネクタで「入力の入口（WS/REST decode）」に guard を必須化する。

前提（依存）
	•	先に Task1（UCEL-DECIMAL-IMPL-CORE-001）で ucel-core::decimal::serde::deserialize_decimal_guarded_default が実装されていること。
	•	これにより、コネクタ側は “中央SSOTのguard” を呼べる。

対象（コネクタ）
	•	ucel/crates/ucel-cex-coinbase/src/lib.rs
	•	ucel/crates/ucel-cex-upbit/src/lib.rs
	•	ucel/crates/ucel-cex-binance-options/src/lib.rs
	•	ucel/crates/ucel-cex-deribit/src/lib.rs
	•	ucel/crates/ucel-cex-bitbank/src/lib.rs
	•	ucel/crates/ucel-cex-binance-coinm/src/lib.rs
	•	ucel/crates/ucel-cex-htx/src/lib.rs
	•	ucel/crates/ucel-cex-sbivc/src/lib.rs

適用（コード反映工程で実施）

このタスクは docs-only で “適用用patch” を固定する。
反映時は repo ルートで順に実行:
	1.	Coinbase:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.coinbase.patch
	2.	Upbit:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.upbit.patch
	3.	Binance Options:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-options.patch
	4.	Deribit:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.deribit.patch
	5.	Bitbank:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.bitbank.patch
	6.	Binance Coin-M:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.binance-coinm.patch
	7.	HTX:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.htx.patch
	8.	SBIVC:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CEX-003.sbivc.patch

最後に:
	•	cargo test -p ucel-cex-coinbase -p ucel-cex-upbit -p ucel-cex-binance-options -p ucel-cex-deribit -p ucel-cex-bitbank -p ucel-cex-binance-coinm -p ucel-cex-htx -p ucel-cex-sbivc

このタスク時点の“より完璧”チェック（含めるべき追加改修が無いか）
	•	WS decode 入口で f64 を排除（少なくとも主要 price/qty）
	•	serde 入口で guard を必須化（負値/scale/0等の侵入防止）
	•	parse::<f64>() を撤去（binance-options）
	•	Decimal配列（Deribit orderbook）も guard 経由にするため wrapper を導入
	•	“tick/step適用の最終強制” は発注境界（Execution/Order builder）で実施すべき → 次の工程（Execution側）で必須

このタスク時点の再点検メモ
	•	CEX各crateの Cargo.toml に ucel-core の decimal serde を使うための依存不足が無いか？
	•	既に ucel_core::Decimal を使っているので、追加不要の想定
	•	binance-options の parse_num 伝播修正が必要 → 反映工程でコンパイルエラー箇所を Decimal 化
	•	Deribit/HTX の wrapper 型導入で変換箇所が漏れていないか → 反映工程でコンパイルエラーを起点に修正
	•	tick/step の最終強制（発注境界） → Execution側タスクで必須
