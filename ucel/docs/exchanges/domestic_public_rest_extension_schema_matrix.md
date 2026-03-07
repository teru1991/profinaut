# Domestic Public REST Extension Schema Matrix (JP)

| venue | operation_id | category | schema_version | payload_type | source_surface |
|---|---|---|---|---|---|
| bitbank | `crypto.public.rest.market.circuit-break-info` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bitbank | `crypto.public.rest.market.transactions` | vendor_public_misc | 1.0.0 | time_series | vendor_public_call_typed |
| bitflyer | `crypto.public.rest.board.get` | vendor_public_instrument_rule | 1.0.0 | object | vendor_public_call_typed |
| bitflyer | `crypto.public.rest.boardstate.get` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bitflyer | `crypto.public.rest.chats.get` | vendor_public_reference | 1.0.0 | time_series | vendor_public_reference_typed |
| bitflyer | `crypto.public.rest.executions.get` | vendor_public_misc | 1.0.0 | time_series | vendor_public_call_typed |
| bitflyer | `crypto.public.rest.health.get` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bitflyer | `fx.public.rest.board.get` | vendor_public_instrument_rule | 1.0.0 | object | vendor_public_call_typed |
| bitflyer | `fx.public.rest.boardstate.get` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bitflyer | `fx.public.rest.executions.get` | vendor_public_misc | 1.0.0 | time_series | vendor_public_call_typed |
| bitflyer | `fx.public.rest.health.get` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bittrade | `public.rest.common.currencys.get` | vendor_public_reference | 1.0.0 | array | vendor_public_reference_typed |
| bittrade | `public.rest.common.timestamp.get` | vendor_public_status | 1.0.0 | enum_like_object | vendor_public_status_typed |
| bittrade | `public.rest.market.detail.merged.get` | vendor_public_instrument_rule | 1.0.0 | object | vendor_public_call_typed |
| coincheck | `coincheck.rest.public.exchange.orders.rate.get` | vendor_public_reference | 1.0.0 | object | vendor_public_reference_typed |
| coincheck | `coincheck.rest.public.order_books.get` | vendor_public_instrument_rule | 1.0.0 | object | vendor_public_call_typed |
