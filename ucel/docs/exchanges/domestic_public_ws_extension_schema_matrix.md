# Domestic Public WS Extension Schema Matrix

| venue | operation_id | category | payload_type | schema_version |
| --- | --- | --- | --- | --- |
| bitbank | `crypto.public.ws.market.circuit-break-info` | vendor_public_status_stream | enum_like_object | 1.0.0 |
| bitbank | `crypto.public.ws.market.transactions` | vendor_public_misc_stream | event_series | 1.0.0 |
| bitflyer | `crypto.public.ws.board` | vendor_public_instrument_rule_stream | snapshot_and_delta | 1.0.0 |
| bitflyer | `crypto.public.ws.board_snapshot` | vendor_public_instrument_rule_stream | object | 1.0.0 |
| bitflyer | `crypto.public.ws.executions` | vendor_public_misc_stream | event_series | 1.0.0 |
| bitflyer | `fx.public.ws.board` | vendor_public_instrument_rule_stream | snapshot_and_delta | 1.0.0 |
| bitflyer | `fx.public.ws.board_snapshot` | vendor_public_instrument_rule_stream | object | 1.0.0 |
| bitflyer | `fx.public.ws.executions` | vendor_public_misc_stream | event_series | 1.0.0 |
| bittrade | `public.ws.market.bbo` | vendor_public_reference_stream | object | 1.0.0 |
| bittrade | `public.ws.market.detail` | vendor_public_misc_stream | enum_like_object | 1.0.0 |
