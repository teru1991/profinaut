# Verification: UCEL Decimal Policy / OrderGate (SSOT)

この文書は「Decimal運用ルールをUCEL内で固定」が **本当に実装完了**したかを判定するための、検査手順（DoD検証）である。
CIに組み込む場合も、手動検査でもそのまま使える。

---

## 1) 前提（最低条件）
- `ucel-core` に以下が存在すること:
  - `ucel_core::decimal::{guard,policy,tick_step,serde}`
  - `ucel_core::order_gate::OrderGate`
- 代表的な発注経路（少なくとも1つ）が OrderGate を必ず通すこと
- CEXコネクタのWS/REST入力で “主要な price/qty” が f64 で残っていないこと
- 入口で serde/parse が guard を通すこと（value-class guard を推奨）

---

## 2) 実装完了チェック（コマンド集）
### 2.1 ビルド＆テスト（最低）
- `cargo test -p ucel-core`
- `cargo test -p ucel-symbol-core`
- CEX（存在するもの全て）:
  - `cargo test -p ucel-cex-coinbase -p ucel-cex-upbit -p ucel-cex-binance-options -p ucel-cex-deribit -p ucel-cex-bitbank -p ucel-cex-binance-coinm -p ucel-cex-htx -p ucel-cex-sbivc`
  - （krakenがある場合） `cargo test -p ucel-cex-kraken`

### 2.2 f64残存監査（事故源の摘出）
目的：価格/数量/残高/約定などの “取引に関わる値” が f64 のまま残っていないことを確認する。

推奨コマンド（repo root）:
- `rg -n ":\s*f64\b" ucel/crates | rg -n "(price|qty|amount|volume|bid|ask|free|locked|balance|mark|index|last|size)"`
期待：結果が 0 件、または “無害（timestamp等）” のみであること。  
無害判定が必要な場合は、該当行にコメントで根拠を残す（将来の誤検出防止）。

### 2.3 無検証 parse/serde 監査（不正値侵入の摘出）
目的：Decimalを `parse::<Decimal>()` で無検証に通していないこと、serdeの入口で guard を通していることを確認。

推奨コマンド:
- `rg -n "parse::<Decimal>$begin:math:text$$end:math:text$" ucel/crates`
- `rg -n "deserialize_with\s*=\s*\".*decimal" ucel/crates`
期待：
- parse は “ガード付き関数” に集約されていること（少なくとも負値拒否、理想は value-class guard）
- serde は `ucel_core::decimal::serde::{deserialize_decimal_execution|balance|observation}` を使用していること

### 2.4 OrderGate 未適用監査（tick/step責務化の確認）
目的：発注（create/add_order 等）を行う箇所が OrderGate を通していることを確認。

推奨コマンド:
- `rg -n "(add_order|create_order|place_order|submit_order|new_order)" ucel/crates/ucel-cex-*`
- `rg -n "OrderGate" ucel/crates/ucel-cex-*`
期待：
- 発注の直前に `OrderGate::validate_limit` または `OrderGate::quantize_limit` が呼ばれること
- tick_size/step_size が filters/catalog から取得され、それを gate に渡していること
  - もし固定値が残っている場合は TODO でなく “上位層で注入する設計” に統一されていること（事故防止のため）

---

## 3) 失敗時の典型原因と対処
### A) コンパイルエラー（Decimal化の伝播漏れ）
- 典型：関数戻り値が f64 のまま、または JSON payload が数値型のまま
- 対処：Decimalに揃える。payloadは **文字列** で送る（取引所API仕様に従う）

### B) 実行時エラー（guardが厳しすぎる/緩すぎる）
- execution で zero を拒否するのは原則正しい
- 観測/残高は value-class guard を使い分ける
- max_abs で弾かれる場合：
  - 異常値侵入の可能性をまず疑う
  - 正常値で弾かれるなら上限設計の見直し（SSOT更新＋テスト更新が必要）

### C) 注文拒否（tick/step違反）
- tick/step の取得値が誤っているか、OrderGate を通っていない
- quantize mode（BUY=Ceil/SELL=Floor/Qty=ToZero）が不適切

---

## 4) 完了判定（チェックボックス）
- [ ] ucel-core の decimal/policy/tick_step/serde/guard が存在し、公開されている
- [ ] OrderGate が存在し、発注直前に必ず呼ばれている
- [ ] 主要コネクタの price/qty が f64 で残っていない
- [ ] parse/serde の入口で不正値が拒否される（value-class guard が望ましい）
- [ ] cargo test がすべて通る
- [ ] 本runbookの監査コマンドで重大な残課題が出ない
