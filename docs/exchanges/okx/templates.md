# Templates (001F)

## REST row template
| id | section | visibility | instType | method | path_or_doc | summary | auth | rate_limit | source_url | status |
|---|---|---|---|---|---|---|---|---|---|---|
| okx.rest.example | <overview/public/private> | <public/private> | <SPOT/SWAP/FUTURES/OPTION/ALL> | <GET/POST/...> | <endpoint or doc-ref> | <summary> | <none/apiKey/sign> | <rule> | <official url> | <verified/blocked> |

## WS row template
| id | ws_area | visibility | instType | channel_or_doc | summary | auth | source_url | status |
|---|---|---|---|---|---|---|---|---|
| okx.ws.example | <overview/public/private/business> | <public/private> | <SPOT/SWAP/FUTURES/OPTION/ALL> | <channel or doc-ref> | <summary> | <none/apiKey/sign/mixed> | <official url> | <verified/blocked> |

## Diff row template
| date(YYYY-MM-DD) | change_type | affected_area | market_scope | summary | action_required | source_url |
|---|---|---|---|---|---|---|
| 2026-01-01 | added/changed/deprecated | <rest/ws/auth/rate-limit/error> | <SPOT/SWAP/FUTURES/OPTION/ALL> | <official changelog text summary> | <required action> | <official changelog url> |
