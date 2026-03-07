# Error Catalog SSOT (A-1 Standard Error Envelope)

## Scope
This document is the SSOT for `code` / `reason_code` used in the standard error envelope.

## Catalog
| code | reason_code | default_http_status | kind | severity | retryable | explanation | example |
|---|---|---:|---|---|---|---|---|
| PLATFORM_VALIDATION_ERROR | REQUEST_BODY_INVALID | 422 | validation_error | warn | false | Request body/query/path validation failed. | malformed JSON body |
| PLATFORM_BAD_REQUEST | REQUEST_INVALID | 400 | client_error | warn | false | Semantically invalid request. | invalid enum/value |
| PLATFORM_UNAUTHORIZED | AUTH_REQUIRED | 401 | auth_error | warn | false | Missing/invalid authentication. | missing token |
| PLATFORM_FORBIDDEN | PERMISSION_DENIED | 403 | permission_error | warn | false | Caller lacks required permissions. | RBAC denied |
| PLATFORM_NOT_FOUND | RESOURCE_NOT_FOUND | 404 | client_error | info | false | Resource was not found. | unknown id |
| PLATFORM_CONFLICT | STATE_CONFLICT | 409 | conflict_error | warn | false | Current resource state conflicts with requested action. | duplicate key, invalid transition |
| PLATFORM_RATE_LIMITED | TOO_MANY_REQUESTS | 429 | rate_limit_error | warn | true | Rate limit/backpressure active. | burst over quota |
| PLATFORM_UPSTREAM_TIMEOUT | UPSTREAM_TIMEOUT | 504 | timeout_error | error | true | Upstream dependency timeout. | upstream read timed out |
| PLATFORM_UPSTREAM_UNAVAILABLE | UPSTREAM_UNAVAILABLE | 503 | unavailable_error | error | true | Upstream dependency unavailable. | DNS/connect failure |
| PLATFORM_INTERNAL_ERROR | UNHANDLED_EXCEPTION | 500 | internal_error | critical | false | Unhandled internal exception normalized by handler. | unexpected runtime exception |
| PLATFORM_CONTRACT_VIOLATION | RESPONSE_SCHEMA_INVALID | 500 | internal_error | error | false | Response schema/contract violation. | schema validation failure |

## Rules
- `code` is a stable logical category.
- `reason_code` is a more specific reason.
- New entries must be accompanied by contract/schema tests.
