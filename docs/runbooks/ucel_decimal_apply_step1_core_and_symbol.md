# Apply Step 1 (SSOT): ucel-core + ucel-symbol-core + OrderGate (Decimal Core)

この手順は “実コード反映工程” 用。docs-onlyタスクで固定した patch を ucel/** に適用し、コンパイル修正→テスト成功までを一本道にする。

## 0) 事前確認
- 現在のブランチで作業して良いこと（実コード変更を含むため）
- 依存: 以下の patch が docs/patches/ucel に存在すること
  - UCEL-DECIMAL-IMPL-CORE-001.patch
  - UCEL-DECIMAL-IMPL-SYMBOL-002.patch
  - UCEL-DECIMAL-REMAIN-1-001.patch
  - UCEL-DECIMAL-REMAIN-2-001.patch

## 1) 適用順（この順番固定）
1) ucel-core の decimal/value 追加 + f64除去:
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-CORE-001.patch
2) ucel-symbol-core の委譲:
   - git apply docs/patches/ucel/UCEL-DECIMAL-IMPL-SYMBOL-002.patch
3) OrderGate（発注前最終強制API）追加:
   - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-1-001.patch
4) max_abs（桁あふれ上限） + 値クラス別 serde:
   - git apply docs/patches/ucel/UCEL-DECIMAL-REMAIN-2-001.patch

※ もし git apply が失敗したら:
- git apply --reject --whitespace=fix <patch>
- 生成された .rej を開き、同じ変更を手で入れる（差分は patch に書かれている通り）

## 2) 直後にやるコマンド（ここでエラーを出し切る）
- cargo test -p ucel-core

期待:
- まず ucel-core が通る。通らない場合は下の “典型修正” を参照。

## 3) 典型修正（迷いゼロのための固定手順）
### 3.1 Decimal::from_str_exact が未import/未使用
エラー例:
- cannot find function `from_str_exact`
対処:
- 対象ファイルで use ucel_core::Decimal; を使っているなら `Decimal::from_str_exact("...")` は使えるはず
- それでも無い場合は `use rust_decimal::Decimal;` と衝突している可能性
  - 方針: ucel_core::Decimal を優先（SSOT）
  - rust_decimal::Decimal の import を削る/名前を変える

### 3.2 serde_json が dependencies に無い
- ucel-core の decimal/serde.rs は serde_json::Number を使う
- Cargo.toml の [dependencies] に serde_json.workspace = true があることを確認

### 3.3 FillEvent/Balance の f64 -> Decimal 伝播
Task1 patch で ucel-core の型が変わるため、下流でコンパイルエラーが出る可能性がある。
この Step1 では “ucel-core 単体の test” を通すだけ。
下流の修正は Step2/3 でまとめて行う。

## 4) 次に ucel-symbol-core を通す
- cargo test -p ucel-symbol-core

期待:
- RoundingStrategy の参照が残っていない
- ucel_core::decimal の import が解決する

もし失敗したら:
- rust_decimal を直接 import している箇所を探す:
  - rg -n "rust_decimal::" ucel/crates/ucel-symbol-core
  - 0件にする（normalize/format は Decimal のAPIだけで足りる）

## 5) 完了条件（Step1）
- cargo test -p ucel-core が成功
- cargo test -p ucel-symbol-core が成功
