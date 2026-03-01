# UCEL OrderGate Spec (SSOT)

## 1. 目的
OrderGate は “tick/stepの適用をUCELが責務化” するための最終安全装置である。
入力が安全でも、発注直前で tick/step を確実に通さなければ事故（注文拒否/意図しない価格数量/会計破損）が起きる。

## 2. SSOT（固定）
- 発注に使う price/qty は、生Decimalやf64のまま送らない
- 発注直前に必ず OrderGate を通す
- OrderGate は次を担う:
  - Decimal Guard（負値/0/scale/max_abs 等）
  - tick/step の validate/quantize
  - side別の推奨quantize（BUY=Ceil, SELL=Floor, Qty=ToZero）

## 3. API（概念）
- validate_limit(price, qty, tick, step) -> (Price, Qty) or Err
  - 既に tick/step 整合が取れているはずの経路で使用
- quantize_limit(side, raw_price, raw_qty, tick, step, price_mode, qty_mode) -> (Price, Qty) or Err
  - upstreamが“生値”を渡す経路で使用
  - 量子化後に再validate（defense-in-depth）

## 4. 入力（OrderGateが要求するもの）
- tick_size / step_size は市場/シンボルのfiltersから取得されるべき
- OrderGate は「filtersの取得」そのものは責務外（上位層が渡す）
  - ただし渡された tick/step の unit<=0 は必ず Err

## 5. 例外ポリシー（固定）
- 観測値（ticker/mark/index）は observation policy を使うが、
  発注境界では execution policy（strict）を必ず使う。
- “ゼロ許容” は値クラス（balance/observation）でのみ許可し、executionでは原則禁止。

## 6. 運用ルール
- tick/step違反が出た場合:
  - まず filters（tick/step）取得値を確認
  - 次に量子化 mode が side/目的に合っているか確認
  - その上で upstream が生値を誤って渡していないか確認
