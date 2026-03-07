# Domestic Public WS Extension Usage

Use `DomesticPublicWsExtensionFacade` from SDK.

- `vendor_public_subscribe_typed(operation_id, params)`
- `vendor_public_reference_subscribe_typed(operation_id, params)`
- `vendor_public_status_subscribe_typed(operation_id, params)`

Preview supported operations:
- `preview_domestic_public_ws_extension_support()`

Operations currently fixed in v1.0.0 include:
- crypto.public.ws.market.circuit-break-info
- crypto.public.ws.market.transactions
- crypto.public.ws.board
- crypto.public.ws.board_snapshot
- crypto.public.ws.executions
- fx.public.ws.board
- fx.public.ws.board_snapshot
- fx.public.ws.executions
- public.ws.market.bbo
- public.ws.market.detail
