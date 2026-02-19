# REST API Catalog (Official) — Coinbase

Order:
1) Advanced Trade Public REST
2) Advanced Trade Private REST
3) Exchange Public REST
4) Exchange Private REST
5) INTX Public REST
6) INTX Private REST
7) Other

## Advanced Trade Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| advanced.crypto.public.rest.reference.introduction | GET | https://api.coinbase.com | /api/v3/brokerage/* | v3 | Advanced Trade public REST catalog root | none | object<k:string,v:any> | object<k:string,v:any> | object | object | endpoint specific; see official reference pages | object | documented in official REST reference | API reference introduction row used as root entry; endpoint-level rows should be expanded from this root in subsequent refreshes | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |

## Advanced Trade Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| advanced.crypto.private.rest.reference.introduction | GET | https://api.coinbase.com | /api/v3/brokerage/* | v3 | Advanced Trade private REST catalog root | jwt | object<k:string,v:any> | object<k:string,v:any> | object | object | endpoint specific; see official reference pages | object | documented in official REST reference | official Advanced Trade private APIs require authenticated requests; details are in per-endpoint pages under this reference root | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |

## Exchange Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| exchange.crypto.public.rest.reference.introduction | GET | https://api.exchange.coinbase.com | /* | v1 | Exchange public REST catalog root | none | object<k:string,v:any> | object<k:string,v:any> | object | object | endpoint specific; see official reference pages | object | documented in official REST reference | Exchange welcome + REST introduction are canonical roots for public resources | https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/introduction |

## Exchange Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| exchange.crypto.private.rest.reference.introduction | GET | https://api.exchange.coinbase.com | /* | v1 | Exchange private REST catalog root | signed | object<k:string,v:any> | object<k:string,v:any> | object | object | endpoint specific; see official reference pages | object | documented in official REST reference | authenticated trading/account endpoints are documented under this official reference section | https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/introduction |

## INTX Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| intx.crypto.public.rest.reference.welcome | GET | not_applicable | not_applicable | not_applicable | INTX REST public discoverability root | none | object<k:string,v:any> | object<k:string,v:any> | object | object | not_applicable | object | not_applicable | INTX welcome page is the navigation root used to determine REST availability/pages | https://docs.cdp.coinbase.com/international-exchange/introduction/welcome |

## INTX Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| intx.crypto.private.rest.reference.welcome | GET | not_applicable | not_applicable | not_applicable | INTX REST private discoverability root | api-key | object<k:string,v:any> | object<k:string,v:any> | object | object | not_applicable | object | not_applicable | INTX welcome page is used as the official source root for private REST documentation reachability | https://docs.cdp.coinbase.com/international-exchange/introduction/welcome |

## Other
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.other.public.rest.docs.root | GET | https://docs.cdp.coinbase.com | / | n/a | documentation root | none | object | object | object | object | n/a | object | n/a | cross-surface reference root | https://docs.cdp.coinbase.com |
