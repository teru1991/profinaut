# Domestic Public WS Runtime Matrix

This matrix fixes runtime policy visibility for canonical WS channels and 009E pending vendor-extension channels.

| venue | public_id | class | ack_mode | integrity_mode | heartbeat | notes |
| --- | --- | --- | --- | --- | --- | --- |
| bitbank | crypto.public.ws.market.depth-diff | canonical_core | implicit_observation | sequence_and_checksum | required | orderbook delta stream |
| bitbank | crypto.public.ws.market.depth-whole | canonical_core | implicit_observation | snapshot_only | required | orderbook snapshot stream |
| bitbank | crypto.public.ws.market.ticker | canonical_core | implicit_observation | none | optional | ticker stream |
| bitbank | crypto.public.ws.market.transactions | vendor_public_extension | implicit_observation | sequence_only | required | pending_009e |
| bitbank | crypto.public.ws.market.circuit-break-info | vendor_public_extension | explicit_ack | none | optional | pending_009e |
| bitflyer | crypto.public.ws.ticker | canonical_core | implicit_observation | none | optional | ticker stream |
| bitflyer | fx.public.ws.ticker | canonical_core | implicit_observation | none | optional | ticker stream |
| bitflyer | crypto.public.ws.board | vendor_public_extension | implicit_observation | sequence_and_checksum | required | pending_009e |
| bitflyer | crypto.public.ws.board_snapshot | vendor_public_extension | implicit_observation | snapshot_only | required | pending_009e |
| bitflyer | crypto.public.ws.executions | vendor_public_extension | implicit_observation | sequence_only | required | pending_009e |
| bitflyer | fx.public.ws.board | vendor_public_extension | implicit_observation | sequence_and_checksum | required | pending_009e |
| bitflyer | fx.public.ws.board_snapshot | vendor_public_extension | implicit_observation | snapshot_only | required | pending_009e |
| bitflyer | fx.public.ws.executions | vendor_public_extension | implicit_observation | sequence_only | required | pending_009e |
| coincheck | coincheck.ws.public.orderbook | canonical_core | implicit_observation | snapshot_only | optional | orderbook stream |
| coincheck | coincheck.ws.public.trades | canonical_core | implicit_observation | none | optional | trades stream |
| gmocoin | crypto.public.ws.orderbooks.update | canonical_core | explicit_ack | sequence_and_checksum | required | orderbook stream |
| gmocoin | crypto.public.ws.ticker.update | canonical_core | explicit_ack | none | required | ticker stream |
| gmocoin | crypto.public.ws.trades.update | canonical_core | explicit_ack | sequence_only | required | trades stream |
| gmocoin | fx.public.ws.orderbooks.update | canonical_core | explicit_ack | sequence_and_checksum | required | orderbook stream |
| gmocoin | fx.public.ws.ticker.update | canonical_core | explicit_ack | none | required | ticker stream |
| gmocoin | fx.public.ws.trades.update | canonical_core | explicit_ack | sequence_only | required | trades stream |
| bittrade | public.ws.market.depth | canonical_core | implicit_observation | sequence_and_checksum | required | orderbook stream |
| bittrade | public.ws.market.kline | canonical_core | implicit_observation | none | optional | candles stream |
| bittrade | public.ws.market.trade.detail | canonical_core | implicit_observation | sequence_only | optional | trades stream |
| bittrade | public.ws.market.bbo | vendor_public_extension | immediate_active | none | optional | pending_009e |
| bittrade | public.ws.market.detail | vendor_public_extension | immediate_active | none | optional | pending_009e |
| sbivc | crypto.public.ws.market_data.orderbook | canonical_core | explicit_ack | sequence_and_checksum | required | orderbook stream |
| sbivc | crypto.public.ws.market_data.ticker | canonical_core | explicit_ack | none | required | ticker stream |
| sbivc | crypto.public.ws.market_data.trades | canonical_core | explicit_ack | sequence_only | required | trades stream |
