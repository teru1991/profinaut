# REST API Catalog (001F)

| id | section | visibility | instType | method | path_or_doc | summary | auth | rate_limit | source_url | status |
|---|---|---|---|---|---|---|---|---|---|---|
| okx.rest.overview | overview | public | ALL | GET | doc-ref | API v5 REST 概要 | none | doc | https://www.okx.com/docs-v5/en/ | blocked |
| okx.rest.auth | overview | private | ALL | GET | doc-ref | REST認証仕様 | apiKey/sign | doc | https://www.okx.com/docs-v5/en/#overview-rest-authentication | blocked |
| okx.rest.errors | overview | public | ALL | GET | doc-ref | エラーコード | none | n/a | https://www.okx.com/docs-v5/en/#error-code | blocked |
| okx.rest.rate-limit | overview | public | ALL | GET | doc-ref | レート制限 | none | n/a | https://www.okx.com/docs-v5/en/#overview-rate-limit | blocked |
| okx.rest.public | public | public | SPOT/SWAP/FUTURES/OPTION | GET | doc-family | Public Data/Market Data 系 | none | per endpoint | https://www.okx.com/docs-v5/en/ | blocked |
| okx.rest.private | private | private | SPOT/SWAP/FUTURES/OPTION | GET/POST | doc-family | Account/Trade/Funding/Sub-account 等 | apiKey/sign | per endpoint | https://www.okx.com/docs-v5/en/ | blocked |
