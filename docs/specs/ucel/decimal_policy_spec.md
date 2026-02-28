UCEL Decimal Policy Spec (SSOT)

1. 目的（事故ゼロのための固定ルール）

UCELは、価格/数量/残高/約定など「取引の意思決定・発注・会計」に関わる数値の表現と変換をSSOTとして固定する。

Decimal導入だけでは不十分であり、以下をUCELの責務として固定する：
	•	丸め規約（用途別・変更禁止）
	•	比較規約（等値判定/ソート/ゼロ判定の固定）
	•	tick/stepの適用（検証と量子化の分離、side別安全丸め）
	•	不正値拒否（負値・0・scale/桁あふれ・tick違反などを型/ガードで拒否）

このSSOTは、CEXコネクタ/シンボル仕様/Execution（唯一の発注出口）で必ず参照される。

⸻

2. 用語
	•	Decimal: rust_decimal による10進固定小数
	•	tick_size: 価格が従う最小刻み（例 0.01）
	•	step_size: 数量が従う最小刻み（例 0.001）
	•	validate: 規約に適合しているか“厳格に検証”する（違反はErr）
	•	quantize: 規約へ“量子化”する（Floor/Ceil/Nearest/ToZero等）
	•	guard: 不正値の拒否（負値、0、scale、桁、上限/下限など）
	•	SSOT: Single Source of Truth。UCELが定め、散在禁止

⸻

3. スコープ（UCELが責務化する境界）

3.1 UCELが必ず保証すること（MUST）
	•	外部入力（WS/HTTP/JSON）で受ける数値は guardを通過していること
	•	発注に使用される価格/数量は tick/stepに整合していること
	•	“境界（発注直前/外部I/O）”では 生f64を禁止し、Decimal（またはNewtype）に統一すること

3.2 UCELが保証しないこと（SHOULD / OUT-OF-SCOPE）
	•	戦略固有の期待値計算や統計処理の内部実装はUCELの外（ただし外部I/Oや発注境界はUCEL責務）

⸻

4. 不正値拒否（Guard: MUST）

4.1 デフォルト拒否条件（固定）
	•	allow_negative = false（負値禁止）
	•	allow_zero = false（ゼロ禁止：値クラスにより例外あり）
	•	max_scale = 18（小数桁上限）
	•	max_abs（絶対値上限）は必要に応じて導入（将来拡張）

4.2 値クラス別の例外（固定方針）
	•	Ticker/Mark等の観測値：
	•	allow_zero = true を許可する場合がある（市場停止/初期値の0を許容）
	•	Order/Executionに使う Price/Qty：
	•	allow_zero = false（原則固定）
	•	Balance:
	•	allow_zero = true（残高0は自然）

⸻

5. 丸め規約（Rounding: MUST）

5.1 用途別の固定規約
	•	Price:
	•	round half up 相当（MidpointAwayFromZero）をSSOTとして採用
	•	Qty:
	•	安全側（ToZero）を基本とする
	•	ただし最終的な量子化は tick/step の quantize mode によって明示される

5.2 変更管理（禁止事項）
	•	どの丸めを使うかを、各adapterや各モジュールで勝手に変えない
	•	新しい規約が必要な場合は UCELのSSOT更新としてのみ許可（仕様・実装・テストを同時変更）

⸻

6. 比較規約（Comparison: MUST）
	•	Decimalは “表現の違い（scale違い）” があり得るため、
	•	等値：a == b を基本（正規化が必要なケースは policy が担保）
	•	順序：cmp は Decimalの全順序に従う
	•	例外：許容誤差を用いる比較（epsilon）は UCELの発注境界では禁止
（統計/分析用途のみ、UCEL外で実施）

⸻

7. tick/step（MUST）

7.1 validate（厳格検証）
	•	value / unit の結果が整数であること（fract==0）
	•	違反は Err（勝手に丸めない）

7.2 quantize（量子化）
	•	mode:
	•	Floor / Ceil / Nearest / ToZero を提供
	•	side別推奨（固定ガイド）
	•	BUYの価格: Ceil（約定確率を上げる） or Nearest（戦略に依存）
	•	SELLの価格: Floor or Nearest
	•	Qty: ToZero（過剰発注を避ける）を基本

⸻

8. どこで弾くか / どこでフラグにするか（例外設計）

8.1 MUST（弾く）
	•	発注に使う Price/Qty の不正値（負値/0/tick違反/step違反/scale超過）は 必ずErrで拒否
	•	外部I/Oのdeserialize時点で拒否できるなら、その時点で拒否（fail-fast）

8.2 MAY（フラグにする）
	•	観測系イベント（Ticker/Mark）の “一時的な0” を許容しつつ Quality を下げる
	•	ただし “発注へ流す前” に必ず再検証（validate）して拒否する

⸻

9. なぜUCEL側で必須か（根拠）
	•	価格/数量の規約を各adapterに散らすと、丸め方向やtick適用が不一致になり、
Execution（唯一の発注出口）で 注文拒否・意図しない価格/数量・残高/PnLの破損が発生する。
	•	Domain Map運用でも crosscut（安全/診断/再現性）を参照する設計のため、
Decimal Policy を docs に固定し、実装とテストを同じSSOTに追随させるのが最も安全。

⸻

10. 参照（実装SSOTの位置）
	•	ucel-core: Decimal Policy / Guard / TickStep / Newtypes の正本
	•	ucel-symbol-core: ucel-core の policy に委譲（独自丸め禁止）
	•	CEX connectors: 入力の guard / 出力の tick/step 適用（f64禁止）
