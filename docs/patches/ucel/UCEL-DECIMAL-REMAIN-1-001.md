# UCEL-DECIMAL-REMAIN-1-001 — Order Gate (final tick/step enforcement) Patch SSOT

## 目的（この段階で “責務化” を完了）
これまでのDecimal SSOT（guard/policy/tick-step/newtypes）だけでは、「tick/stepの適用をUCELが責務化」の核心が未完になる。
理由：入力が安全でも、発注直前に tick/step を必ず通さないと事故が起きるため。

本パッチは ucel-core に “Order Gate（発注前ゲート）” を追加し、以下を満たす：
- Price/Qty を生Decimalで発注に流さない（必ず gate 経由）
- tick/step を **validate/quantize** の規約で適用（side別 mode を明示）
- 値クラス別 policy を追加（観測/残高/発注のゼロ許容など）

## 対象（実コード変更）
### 追加（NEW）
- ucel/crates/ucel-core/src/order_gate/mod.rs
- ucel/crates/ucel-core/src/order_gate/gate.rs

### 更新（MOD）
- ucel/crates/ucel-core/src/lib.rs（pub mod order_gate、re-export）
- ucel/crates/ucel-core/src/decimal/policy.rs（値クラス別 policy constructor を追加）

## 適用（コード反映工程で実施）
- repoルートで:
  - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-1-001.patch
- その後:
  - cargo test -p ucel-core

## 重要（このタスクで “一緒にやるべき” 追加改修チェック）
- [x] 値クラス別 policy を提供（観測/残高/発注）
- [x] tick/step の最終強制APIを用意（validate と quantize を両方用意）
- [x] エラー型を明確化（GateError）
- [ ] “市場フィルタの取得（tick/step/min_notional等）” は上位層に依存するため、ここでは Gate に「必要な入力」を要求する形に固定（次タスクで各CEXへ接続）
