UCEL-DECIMAL-A-CORE-001 (ucel-core) Patch SSOT

対象
	•	ucel/crates/ucel-core/src/types.rs
	•	ucel/crates/ucel-core/src/lib.rs

ゴール
	•	“canonical numeric type” のSSOTを崩さず、発注/会計に繋がる f64 を排除する
	•	FillEvent.price/qty と Balance.free/locked を Decimal に統一する（最低限）
	•	将来の decimal policy/newtype 公開に備え、lib.rs の re-export を整理する

適用（コード反映工程で実施）
	•	このリポジトリのルートで:
	•	git apply docs/patches/ucel/UCEL-DECIMAL-A-CORE-001.patch
	•	その後:
	•	cargo test -p ucel-core
