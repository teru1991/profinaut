UCEL-DECIMAL-IMPL-CORE-001 — ucel-core Decimal Policy Implementation Patch (SSOT)

目的

このパッチは「Decimal導入だけでは不十分」を解消するため、ucel-core に以下を 実コードとして追加する。
	•	丸め規約（policy）
	•	比較の前提（guard/normalizeの考え方）
	•	tick/step の validate/quantize
	•	不正値拒否（負値、ゼロ、scale超過、桁/絶対値上限）
	•	serde入力のガード（string/numberを受けてguard）
	•	型（Newtype）: Price/Qty/Notional/TickSize/StepSize（境界で生Decimalを避ける準備）

また、ucel-core/src/lib.rs の事故源（FillEvent.price/qty と Balance.free/locked が f64）を Decimal に置換して、精度欠落を防ぐ。

対象（実際に変更されるファイル）

追加（NEW）
	•	ucel/crates/ucel-core/src/decimal/mod.rs
	•	ucel/crates/ucel-core/src/decimal/policy.rs
	•	ucel/crates/ucel-core/src/decimal/guard.rs
	•	ucel/crates/ucel-core/src/decimal/tick_step.rs
	•	ucel/crates/ucel-core/src/decimal/serde.rs
	•	ucel/crates/ucel-core/src/value/mod.rs
	•	ucel/crates/ucel-core/src/value/price.rs
	•	ucel/crates/ucel-core/src/value/qty.rs
	•	ucel/crates/ucel-core/src/value/notional.rs
	•	ucel/crates/ucel-core/src/value/tick_step.rs

更新（MOD）
	•	ucel/crates/ucel-core/Cargo.toml（serde_json を dependencies に追加）
	•	ucel/crates/ucel-core/src/lib.rs（decimal/value module追加、f64→Decimal置換）
	•	ucel/crates/ucel-core/src/types.rs（コメント整備：policy導入の明示）

適用（後続 “コード反映” タスクで実施）
	•	リポジトリルートで:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CORE-001.patch
	•	その後:
	•	cargo test -p ucel-core

重要（設計上の安全）
	•	この段階では、互換性確保のため FillEvent/Balance は Decimal に置換する（Newtypeへ全面移行は後続タスクで段階実施）。
	•	serdeのguardは「負値/scale/0」の拒否を最小限保証。tick/stepは “発注境界” で適用する（後続タスク）。

再点検メモ（このタスク時点）
	•	serde_json を dependencies に追加する必要はあるか？ → 必要（decimal/serde.rs が Number を使う）
	•	ucel-core 以外への影響（コンパイルエラー源）は何か？ → FillEvent/Balance の型変更が波及する
	•	tick/step の unit <= 0 ガードはあるか？ → ある
	•	“桁あふれ” はどう担保するか？ → max_abs を Option で提供。上限値の具体化は後続タスクで市場/ドメイン要件に合わせて決める
	•	tick/step の Nearest の tie-break は固定されているか？ → 固定（0.5はゼロから遠ざける）
	•	例外（Balanceはゼロ許容）の扱いは？ → policy.allow_zero を値クラス別に分けるのが最終形。現段階では “デフォルト” を固定し、値クラス別policy導入は後続へ
