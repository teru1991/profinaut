# UCEL Hub/Registry Surface v1

## Purpose
UCEL の Hub/Registry/Invoker を、workspace の CEX family を横断する単一入口として固定する。

## Logical exchange unit
- Hub/Registry の列挙単位は **logical exchange/family id**（canonical kebab-case）とする。
- `ExchangeId::all()` と registry registration table は一致しなければならない。
- alias は互換入力のみ許可し、出力は canonical id を返す。

## Minimum Hub surface
- `list_exchanges()`
- `exchange_exists(exchange: &str)`
- `capabilities(exchange)`
- `list_catalog_entries(exchange)`
- `resolve_rest(exchange, op)`
- `resolve_ws(exchange, channel)`

## Minimum Invoker surface
- `list_operations(exchange)`
- `list_ws_channels(exchange)`
- unknown exchange/op/channel で安定エラーを返す

## Integration rule
- source of truth は registration table（exchange id, alias, catalog include, crate family）。
- crate が存在し catalog がある family は Hub/Registry/Invoker から到達可能であること。
- 未整備 family を黙って隠蔽せず、少なくとも列挙可能状態にする。

## Canonical IDs (v1)
- `binance`, `binance-usdm`, `binance-coinm`, `binance-options`
- `bitbank`, `bitflyer`, `bitget`, `bithumb`, `bitmex`, `bittrade`
- `bybit`, `coinbase`, `coincheck`, `deribit`, `gmocoin`, `htx`, `kraken`, `okx`, `sbivc`, `upbit`
