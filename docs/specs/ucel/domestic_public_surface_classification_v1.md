# UCEL Domestic Public Surface Classification v1

## Classes
- `canonical_core`
  - `get_ticker`, `get_trades`, `get_orderbook_snapshot`, `get_candles`
  - 市場データの最小共通面
- `canonical_extended`
  - `list_symbols`, `get_market_meta`, `get_system_status`, `get_asset_status`, `get_network_status`, `get_maintenance_status`, `get_public_derivative_reference`
  - 多くの venue で公開される補助 public 面
- `vendor_public_extension`
  - canonical 化されていないが public で有用な endpoint/channel
  - 例: venue 独自 circuit-break/status detail
- `not_supported`
  - inventory では public evidence はあるが、現行 repo policy で意図的に非対応

## Classification rules
1. market data core は vendor extension に落とさない。
2. canonical name が存在しない public endpoint は `vendor_public_extension` で保持。
3. 一つの `venue + public_id` は必ず 1 class。
4. `vendor_public_call` は canonical surface が無い場合に限定。

## Rationale
- 後続 009B/009C で canonical 実装を先行し、009D/009E で拡張を取りこぼさず実装するため。
- not_supported は policy 起因の明示用途に限定し、unknown 放置を禁止するため。
