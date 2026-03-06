# UCEL SSOT Consistency Gate v1

## Purpose
`catalog` / `coverage` / `coverage_v2` / `ws_rules` の整合を機械検証し、曖昧な drift を CI で fail させる。

## SSOT role split
- `docs/exchanges/*/catalog.json`: Hub/Invoker が expose する REST op / WS channel の正本。
- `ucel/coverage/*.yaml`: legacy coverage（互換ブリッジ、監査証跡）。
- `ucel/coverage_v2/*.yaml`: venue/family/surface support の正本。
- `ucel/crates/ucel-ws-rules/rules/*.toml`: WS runtime rule（entitlement / heartbeat / rate / safety）。

## Canonical naming rules
- canonical venue id は kebab-case (`bitbank`, `binance-usdm` など)。
- family split は coverage_v2 venue (`gmocoin-public`, `gmocoin-private`, `okx-swap` など) を優先。
- alias は明示テーブル以外禁止。

## Semantic rules
- access policy (`public_only/public_private/blocked`) と runtime entitlement (`public_only` 等) は別概念。
- ただし矛盾（例: policy public_only なのに entitlement public_private）は fail。
- `supported/not_supported/strict` は fail-open に使わず、曖昧なら fail。

## Consistency gate rules
1. registry 登録 exchange/family は coverage_v2 に到達可能であること（canonical or alias bridge）。
2. catalog に存在する WS id は coverage_v2 の family id に対応すること。
3. coverage_v2 で WS family が supported な venue は ws_rules に rule file が存在すること。
4. ws_rules の exchange_id は coverage_v2 または registry canonical/alias と整合すること。
5. legacy coverage (`coverage/*.yaml`) は coverage_v2 と scope が矛盾しないこと。

## Exception policy
- 例外は `explicit exception` として docs に列挙し、gate でもキー一致でのみ許容。
- TODO コメントだけでの例外は不可。
