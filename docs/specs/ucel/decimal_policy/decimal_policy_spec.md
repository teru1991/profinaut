UCEL Decimal Policy Spec (SSOT)

Scope
	•	対象: 価格（Price）, 数量（Qty）, Notional など「発注/計算に使う値」全般
	•	UCEL が責務化:
	•	丸め規約（用途別）
	•	比較規約（正規化・等値判定）
	•	tick/step 適用（validate/quantize）
	•	不正値拒否（負値/0/scale/桁/単位違反）

Fixed Rules
	•	Guard:
	•	allow_negative = false（デフォルト）
	•	allow_zero = false（デフォルト：必要な箇所のみ例外）
	•	max_scale = 18（デフォルト）
	•	Rounding:
	•	price: MidpointAwayFromZero（round half up 相当）
	•	qty: ToZero（安全側）
	•	Tick/Step:
	•	validate は strict（違反は必ず Err）
	•	quantize は QuantizeMode で明示（side別の選択を強制）
	•	Serde:
	•	deserialize 時点で guard を通し、不正値は拒否（API/WS入力の事故防止）

Examples
	•	tick=0.01 の price:
	•	100.005 は Nearest で 100.01
	•	100.009 は Floor で 100.00
	•	step=0.001 の qty:
	•	0.0009 は Floor で 0.000（allow_zero=falseなら Err）

⸻
