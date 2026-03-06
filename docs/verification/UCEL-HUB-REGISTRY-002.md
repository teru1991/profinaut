# Verification: UCEL-HUB-REGISTRY-002

## 1) Changed files (`git diff --name-only`)
```bash
docs/specs/ucel/hub_registry_surface_v1.md
docs/status/trace-index.json
docs/verification/UCEL-HUB-REGISTRY-002.md
ucel/crates/ucel-registry/src/hub/mod.rs
ucel/crates/ucel-registry/src/hub/registry.rs
ucel/crates/ucel-registry/src/hub/rest.rs
ucel/crates/ucel-registry/src/hub/ws.rs
ucel/crates/ucel-registry/src/invoker/mod.rs
ucel/crates/ucel-registry/src/invoker/registry.rs
ucel/crates/ucel-registry/src/lib.rs
ucel/crates/ucel-testkit/tests/hub_registry_catalog_coverage.rs
ucel/crates/ucel-testkit/tests/hub_registry_examples_smoke.rs
ucel/docs/registry/hub_registry_surface.md
ucel/examples/hub_list_all_exchanges.rs
ucel/examples/hub_list_exchange_catalogs.rs
```

## 2) What / Why
- Hub/Registry/Invoker の exchange/family 情報を `hub/registry.rs` の registration table に集約し、catalog include の散在を解消した。
- `ExchangeId` を全 logical exchange/family（20 ID）へ拡張し、`all()/as_str()/FromStr(alias対応)` を安定化した。
- Hub に `list_exchanges / exchange_exists / list_catalog_entries` を追加し、最小 surface の可視化を固定した。
- Invoker registry は coverage discover 依存から catalog registration 依存へ移し、全 registered exchange の operation/channel 列挙を安定化した。
- testkit に regression test を追加し、workspace CEX member と registry 到達性の差分を自動検知できるようにした。

## 3) Self-check results
- Allowed-path check OK
  - 実行: `git diff --name-only | awk '...allowlist...'`
  - 結果: 出力なし
- Tests added/updated OK
  - `ucel/crates/ucel-registry/src/hub/registry.rs`（unit tests追加）
  - `ucel/crates/ucel-registry/src/invoker/registry.rs`（unit tests追加）
  - `ucel/crates/ucel-testkit/tests/hub_registry_catalog_coverage.rs`（新規）
  - `ucel/crates/ucel-testkit/tests/hub_registry_examples_smoke.rs`（新規）
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-registry` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test hub_registry_catalog_coverage -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test hub_registry_examples_smoke -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit` => FAIL（既存 fixture 不足: `support_bundle_manifest_fixture_is_sane`, `/workspace/profinaut/fixtures/support_bundle/manifest.json` 読み込み失敗）
  - `cd ucel && cargo test -p ucel-registry -p ucel-testkit` => FAIL（同上の既存 fixture 失敗）
  - `cd ucel && cargo run --example hub_list_all_exchanges` => FAIL（workspace root examples は cargo example target として未公開）
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- Secrets scan OK
  - `rg -n "AKIA|SECRET|PRIVATE KEY|BEGIN RSA" <changed files>`
  - 結果: ヒットなし
- docsリンク存在チェック OK（今回触った docs 内 `docs/` 参照）
  - `rg -n "docs/" docs/specs/ucel/hub_registry_surface_v1.md ucel/docs/registry/hub_registry_surface.md`

## 4) ★履歴確認の証拠（必須）
- 実行コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show <直近コミットSHA>`
  - `git show <hub/mod.rs 最終変更SHA>`
  - `git show <hub/registry.rs 最終変更SHA>`
  - `git blame -w ucel/Cargo.toml`
  - `git blame -w ucel/crates/ucel-registry/src/hub/mod.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/registry.rs`
  - `git blame -w ucel/crates/ucel-registry/src/lib.rs`
  - `git blame -w ucel/crates/ucel-registry/src/invoker/registry.rs`
  - `git blame -w ucel/crates/ucel-registry/src/invoker/mod.rs`
  - `git reflog -n 30`
  - `git merge-base HEAD origin/master`（この clone は remote 未設定のため不可）
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git show <latest merge sha> --stat`
- 主要 SHA:
  - 直近: `39967099`
  - hub/mod.rs 最終変更（着手前）: `39967099`
  - hub/registry.rs 最終変更（着手前）: `58868c79` 系列
  - 直近 merge: `46769647`
- 判定結果:
  - `ucel/Cargo.toml` には 19 個の workspace `ucel-cex-*` member がある一方、Hub `ExchangeId` と include は一部のみで分断が存在していた。
  - `ExchangeId` を enum で持つ方針は維持しつつ、命名規約は canonical kebab-case + alias 受理に固定した。
  - hotspot（`hub/mod.rs`, `hub/registry.rs`, `invoker/mod.rs`）は局所変更を維持し、実データは registration table へ集約した。

### Cargo.toml と ExchangeId/registration table の差分棚卸し
- workspace `ucel-cex-*` members（19）:
  - bitbank, gmocoin, kraken, binance-usdm, bybit, upbit, coinbase, coincheck, binance-coinm, binance-options, bitmex, bitflyer, binance, bittrade, deribit, bitget, htx, sbivc, okx
- Hub registration canonical ids（20）:
  - 上記 19 に加えて `bithumb` を明示（crate/catalog は存在、workspace member 外）
- 対応方針:
  - workspace member 19 はすべて `ExchangeId::all()` に含め、Hub/Invoker から到達可能化
  - `bithumb` は沈黙させず registration へ明示（notes 付き）

### blame から読み取れた意図/制約
- `hub/*` と `invoker/*` は初期実装で少数 venue 前提の固定 table だったため、family split 増加に追随できていなかった。
- `invoker/registry` は coverage discover 由来で strict fixture に引きずられ、hub catalog 到達性と独立した失敗モードを持っていた。
- 本タスクで registration table を共通情報源にし、Hub/Invoker で同一 catalog source を辿れるようにした。

### “不足があったため追加実装した”内容
- 追加実装:
  - `CatalogEntry.auth` に `#[serde(default)]` を付与
- 根拠:
  - 登録拡大後、一部 catalog JSON が `auth` 欠落で parse failure となり registry build が停止したため。
- 対策:
  - 互換性を壊さず default で吸収し、Hub/Invoker 統合テーブルの build を安定化。

## 5) Environment limitation notes
- remote が未設定のため `origin/master` 系コマンド（`merge-base`, `origin/master...HEAD`）は実行不可。
