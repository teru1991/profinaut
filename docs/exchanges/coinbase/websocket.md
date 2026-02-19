# WebSocket Catalog (Official) — Coinbase

Order:
1) Advanced Trade Public WS
2) Advanced Trade Private WS
3) Exchange Public WS
4) Exchange Private WS
5) INTX Public WS
6) INTX Private WS
7) WS Common
8) Other

## Advanced Trade Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| advanced.crypto.public.ws.reference.channels | wss://advanced-trade-ws.coinbase.com | v1 | official channel set (see channel page) | {"type":"subscribe","channel":"<channel>","product_ids":["<product_id>"]} | {"type":"unsubscribe","channel":"<channel>","product_ids":["<product_id>"]} | object | channel-specific payloads documented on official channel page | ping-pong | none | channel coverage and product scope are defined in official WS docs | public market-data channels are documented in the official channels page | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-channels |

## Advanced Trade Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| advanced.crypto.private.ws.reference.guide | wss://advanced-trade-ws-user.coinbase.com | v1 | user/order channels | {"type":"subscribe","channel":"<channel>","jwt":"<token>"} | {"type":"unsubscribe","channel":"<channel>"} | object | channel-specific payloads documented in official Advanced Trade WS guide | ping-pong | jwt | authenticated subscription required for private streams | private/user data streams are covered in official WS guide/overview docs | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket |

## Exchange Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| exchange.crypto.public.ws.reference.overview | wss://ws-feed.exchange.coinbase.com | n/a | official channel set (see Exchange WS docs) | {"type":"subscribe","channels":["<channel>"],"product_ids":["<product_id>"]} | {"type":"unsubscribe","channels":["<channel>"],"product_ids":["<product_id>"]} | object | channel-specific payloads documented in Exchange WS docs | server-push | none | per-channel constraints and sequence semantics are documented on official pages | public Exchange WebSocket feed overview root | https://docs.cdp.coinbase.com/exchange/websocket-feed/overview |

## Exchange Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| exchange.crypto.private.ws.not_applicable.current_scope | not_applicable | not_applicable | not_applicable | not_applicable | not_applicable | object | not_applicable | none | not_applicable | not_applicable | Exchange private WS coverage requires dedicated authenticated-channel pages under Exchange WS docs | https://docs.cdp.coinbase.com/exchange/websocket-feed/overview |

## INTX Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| intx.crypto.public.ws.reference.overview | not_applicable | n/a | official channel set (see INTX WS docs) | object | object | object | channel-specific payloads documented on official INTX WS pages | server-push | none | channel and throughput restrictions are defined by INTX WS docs | INTX WebSocket documentation root row | https://docs.cdp.coinbase.com/international-exchange/websocket-feed/websocket-overview |

## INTX Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| intx.crypto.private.ws.reference.welcome | not_applicable | n/a | private stream discoverability | object | object | object | not_applicable | other | token | not_applicable | INTX welcome page is the official root for authenticated stream documentation reachability | https://docs.cdp.coinbase.com/international-exchange/introduction/welcome |

## WS Common
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.crypto.public.ws.common.protocol | not_applicable | n/a | n/a | n/a | n/a | object | protocol/auth/limits are page-specific across surfaces | other | other | see each surface docs root | common WS behavior differs by surface; use per-surface roots in sources.md | https://docs.cdp.coinbase.com |

## Other
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.other.public.ws.docs.root | https://docs.cdp.coinbase.com | n/a | docs-root | n/a | n/a | object | n/a | none | none | n/a | documentation hub reference row | https://docs.cdp.coinbase.com |
