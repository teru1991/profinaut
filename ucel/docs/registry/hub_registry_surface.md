# Hub / Registry Surface (Operational SSOT)

## Registry table contract
Hub registry は次を持つ registration table を唯一の情報源とする。
- canonical exchange id
- alias list
- catalog JSON include
- crate family 名

## Operational checks
- `ExchangeId::all()` と registration 件数の一致をテストで固定
- 全 registration で catalog parse が成功すること
- Hub から全 exchange を列挙できること
- Invoker から全 exchange の operation/channel が列挙できること

## Catalog visibility
- REST/WS entry 数は exchange ごとに `list_catalog_entries()` で可視化
- 件数ゼロでも registry から消さず、状態を明示して扱う（沈黙禁止）

## Policy interoperability
- capabilities は UCEL-POLICY-001 の venue_access を維持
- Hub/Invoker の resolve 後 gate を壊さない

## Human verification
- `ucel/examples/hub_list_all_exchanges.rs`
- `ucel/examples/hub_list_exchange_catalogs.rs`
