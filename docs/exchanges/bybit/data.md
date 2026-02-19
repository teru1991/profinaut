# Data Distribution Catalog (Official)

Fixed columns:
id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url

| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| bybit.data.market.ws.public | market-data-stream | wss://stream.bybit.com/v5/public/{spot|linear|inverse|option} | json | none | real-time | session-lifetime | topic,type,ts,cs?,data | Public market topics (orderbook/trade/ticker/kline/liquidation/insurance/adl/priceLimit) | https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook |
| bybit.data.account.ws.private | account-stream | wss://stream.bybit.com/v5/private | json | none | real-time | session-lifetime | topic,creationTime,id?,data | Private topics (order/execution/position/wallet/greeks/dcp) require auth | https://bybit-exchange.github.io/docs/v5/websocket/private/order |
| bybit.data.trade.ws.trade | order-entry-stream | wss://stream.bybit.com/v5/trade | json | none | request/response + async ack | session-lifetime | op,reqId,header,args,data,retCode,retMsg | WebSocket trade service for create/amend/cancel order operations | https://bybit-exchange.github.io/docs/v5/websocket/trade/guideline |
| bybit.data.market.rest.snapshot | rest-snapshot | https://api.bybit.com/v5/market/* | json | https gzip(opt) | on-demand | request-time | retCode,retMsg,result,time | REST market snapshots used with WS book delta sync | https://bybit-exchange.github.io/docs/v5/market/orderbook |
