# domestic_public_ws_surface_v1

Task: UCEL-DOMESTIC-PUBLIC-WS-009C

## Canonical core WS surface
- subscribe_ticker
- subscribe_trades
- subscribe_orderbook
- subscribe_candles

## Canonical extended WS surface
- subscribe_system_status
- subscribe_maintenance_status
- subscribe_asset_status
- subscribe_network_status
- subscribe_public_derivative_reference
- subscribe_public_funding_reference
- subscribe_public_open_interest_reference

## Classification rules
- Each inventory `api_kind=ws` entry maps to exactly one class.
- `canonical_core` and `canonical_extended` are reachable through typed canonical events.
- `vendor_public_extension` entries are explicitly carried as pending for task 009E.
- `not_supported` may only be used if inventory says `not_supported`.
