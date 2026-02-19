# HTX REST API Catalog (Official Only)

## 1) Spot Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.rest.spot.catalog.index | GET | https://api.htx.com | unknown | unknown | Spot public API catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | category confirmed via official OpenAPI entrypoint; endpoint-level extraction blocked in current environment | https://www.htx.com/en-us/opend/newApiPages/ |

## 2) Spot Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.rest.spot.catalog.index | GET | https://api.htx.com | unknown | unknown | Spot private API catalog index | signed | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | private API category confirmed via official OpenAPI entrypoint | https://www.htx.com/en-us/opend/newApiPages/ |

## 3) Margin Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| margin.public.rest.margin.catalog.index | GET | https://api.htx.com | unknown | unknown | Margin public API catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | margin domain tracked from official OpenAPI navigation root | https://www.htx.com/en-us/opend/newApiPages/ |

## 4) Margin Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| margin.private.rest.margin.catalog.index | GET | https://api.htx.com | unknown | unknown | Margin private API catalog index | signed | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | margin private category tracked from official OpenAPI navigation root | https://www.htx.com/en-us/opend/newApiPages/ |

## 5) Futures Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.public.rest.futures.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Futures public API catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | futures API product confirmed by official support documentation | https://www.htx.com/support/360000188382 |

## 6) Futures Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.private.rest.futures.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Futures private API catalog index | signed | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | futures private domain derived from official futures API access page | https://www.htx.com/support/360000188382 |

## 7) Swap Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| swap.public.rest.swap.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Swap public API catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | swap section expected under official OpenAPI docs nav | https://www.htx.com/en-us/opend/newApiPages/ |

## 8) Swap Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| swap.private.rest.swap.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Swap private API catalog index | signed | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | swap private section expected under official OpenAPI docs nav | https://www.htx.com/en-us/opend/newApiPages/ |

## 9) Options Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.public.rest.options.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Options public API catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | options product evidenced by official options version history page | https://www.htx.com/support/900003014323/ |

## 10) Options Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.private.rest.options.catalog.index | GET | https://api.hbdm.com | unknown | unknown | Options private API catalog index | signed | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | array<object> (opt) | object | unknown | options private scope inferred from official options API history context | https://www.htx.com/support/900003014323/ |

## 11) Earn/Copy/Broker/Other（公式に存在するもののみ。無ければ not_applicable + 根拠URL）

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| earn.private.rest.earn.not_applicable | GET | not_applicable | not_applicable | not_applicable | earn not_applicable | other | object (opt) | object (opt) | object (opt) | object | object | object | unknown | no directly verifiable official earn API page recovered in current environment snapshot | https://www.htx.com/en-us/opend/newApiPages/ |
| copy.private.rest.copy.not_applicable | GET | not_applicable | not_applicable | not_applicable | copy not_applicable | other | object (opt) | object (opt) | object (opt) | object | object | object | unknown | no directly verifiable official copy API page recovered in current environment snapshot | https://www.htx.com/en-us/opend/newApiPages/ |
| broker.private.rest.broker.not_applicable | GET | not_applicable | not_applicable | not_applicable | broker not_applicable | other | object (opt) | object (opt) | object (opt) | object | object | object | unknown | no directly verifiable official broker API page recovered in current environment snapshot | https://www.htx.com/en-us/opend/newApiPages/ |
| other.public.rest.other.catalog.index | GET | https://api.htx.com | unknown | unknown | other domain catalog index | none | object<k:string,v:any> (opt) | object<k:string,v:any> (opt) | object (opt) | object | object | object | unknown | placeholder for additional official categories reachable from OpenAPI nav | https://www.htx.com/en-us/opend/newApiPages/ |
