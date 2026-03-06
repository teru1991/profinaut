# SSOT Consistency Policy (UCEL)

## Enforcement
- testkit の SSOT consistency validator を唯一の判定器として運用する。
- validator は issue を structured diagnostic として出力し、未許容 issue は fail。

## Explicit exceptions (current)
- `gmocoin`: coverage_v2 が `gmocoin-public/private` に split され、registry canonical は `gmocoin`。
  - bridge alias として許容。
- `bitget`: coverage_v2 が `bitget-spot/usdt-futures/coin-futures` に split され、registry canonical は `bitget`。
  - bridge alias として許容。
- `bybit`: coverage_v2 が `bybit-spot/linear/inverse/options` に split され、registry canonical は `bybit`。
  - bridge alias として許容。
- `okx`: coverage_v2 が `okx-spot/swap/futures/option` に split され、registry canonical は `okx`。
  - bridge alias として許容。
- `binance`: coverage_v2 が `binance-spot/usdm/coinm/options` に split され、registry canonical は `binance`。
  - bridge alias として許容。

## Migration bridge
- legacy coverage (`ucel/coverage/*.yaml`) は coverage_v2 と scope 整合のみ検査対象。
- 新規実装の判定は coverage_v2 + ws_rules + registry canonical に寄せる。

## Explicit exceptions (drift gate allowlist)
- Unmigrated coverage_v2 venues (legacy coverage authoritative bridge):
  - bitbank, bitflyer, bithumb, bitmex, coinbase, coincheck, deribit, sbivc, upbit
- Missing ws_rules for supported coverage_v2 families:
  - bitget-coin-futures, bitget-usdc-futures

上記は gate で warning 扱いとし、未列挙の新規不整合は fail する。
