# Data Feed Distribution Catalog (Official)

Fixed columns:
id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url

| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| options.data.ws.market | streaming | wss://nbstream.binance.com/eoptions/ws/<streamName> | json | none | real-time | not stated | options market stream event payloads | Distribution path for market streams in Options WS docs | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.data.rest.market | pull-api | https://eapi.binance.com/eapi/v1/* | json | none | on-demand | not stated | market snapshots/metadata from REST | Market data REST family distribution endpoint namespace | https://developers.binance.com/docs/derivatives/option/market-data/rest-api |
