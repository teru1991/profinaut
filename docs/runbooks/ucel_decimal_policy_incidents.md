# Runbook: UCEL Decimal Policy / OrderGate Incidents

## 目的
Decimal Policy / OrderGate に関する障害（注文拒否、価格数量のズレ、異常値侵入）を最短で切り分ける。

## 1) 症状別の一次切り分け
### A. 注文が取引所に拒否される（tick/step違反）
- OrderGate を通っているか？
  - 通っていない → 発注境界の実装漏れ（最優先で修正）
- tick_size / step_size の取得値は正しいか？
  - シンボルfilters（取引所仕様）と一致するか確認
- quantize mode は正しいか？
  - BUY price: Ceil / SELL price: Floor / Qty: ToZero が基本

### B. 価格/数量が意図せずズレる（丸め方向の不一致）
- “どこで丸めたか” を特定:
  - ucel-core::decimal policy の round/quantize 以外で丸めていないか
- 入口で f64 が残っていないか（精度欠落）
  - 取引所wire/内部イベントの型を確認

### C. 異常値（負値/巨大値）が侵入する
- serde が value-class guard を使っているか:
  - execution: deserialize_decimal_execution
  - observation: deserialize_decimal_observation
  - balance: deserialize_decimal_balance
- max_abs の設定が有効か（ucel-core policy）

## 2) すぐ見るべきポイント（実装観点）
- 発注直前のコードパス:
  - OrderGate の validate/quantize が呼ばれているか
- 取引所filters:
  - tick_size/step_size が正しい値か（型と精度含む）
- 生f64:
  - 価格/数量が f64 で保持される箇所が残っていないか

## 3) 恒久対策（再発防止）
- OrderGate テストを追加/更新し、tick/step違反を必ず検出
- コネクタのwire型は Decimal + serde guard を徹底
- 仕様変更があれば SSOT（docs/specs/ucel/decimal_policy_spec.md）と同時に更新
