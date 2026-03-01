UCEL-DECIMAL-IMPL-SYMBOL-002 — ucel-symbol-core Delegation Patch (SSOT)

目的

ucel-symbol-core に存在する独自の丸め/比較ロジックは、将来 ucel-core 側の Decimal policy と乖離して事故原因になり得る。
よって以下を実施する：
	1.	ucel-symbol-core の Decimal型の正本を ucel-core::Decimal に統一
	2.	丸めは ucel-core::decimal::DecimalPolicy を利用する形へ寄せる（SSOT参照）
	3.	tick/step の validate/quantize を 追加APIとして提供し、上位層が「同じ関数」を呼べるようにする
	4.	ucel-symbol-core は「市場ルール（tick_size/lot_size等）を保持する層」であり、発注境界の最終強制は後続タスクで実施する（Execution/adapter側へ）

対象
	•	ucel/crates/ucel-symbol-core/Cargo.toml
	•	ucel/crates/ucel-symbol-core/src/lib.rs

重要（互換性と安全）
	•	DecimalPolicy::default() は guard のデフォルトが allow_negative=false/allow_zero=false のため、symbol層の丸め用途では拒否が過剰になり得る。
よって本パッチでは symbol 層に限り、内部ヘルパ policy_relaxed() を用意し、
allow_negative=true/allow_zero=true として “丸め/量子化” を安全に利用する（SSOTの型/実装を参照しつつ、値クラス差を吸収）。
	•	tick/step の unit <= 0 は ucel-core 側でErr化されるため、symbol層でも事故らない。

適用（後続 “コード反映” タスクで実施）
	•	リポジトリルートで:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-SYMBOL-002.patch
	•	その後:
	•	cargo test -p ucel-symbol-core

再点検メモ（このタスク時点）
	•	ucel-symbol-core 内に rust_decimal 直接依存が残っていないか？ → Cargo.tomlから削除済み
	•	RoundingStrategy を symbol 層が独自に持っていないか？ → 排除済み（ucel-core policy参照）
	•	normalize_decimal/cmp_decimal が policy と矛盾しないか？ → 矛盾しない（normalizeは表現統一のみ）
	•	tick/step が symbol 側の tick_size/lot_size と整合して呼べるか？ → quantize/validate APIを追加
	•	guard の “ゼロ/負値禁止” が symbol 層で邪魔にならないか？ → policy_relaxed導入で回避
	•	互換性（既存API）を壊していないか？ → 関数は置換（内部実装変更）＋追加APIのみ
