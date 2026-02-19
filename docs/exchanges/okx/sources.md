# Sources (Official only)

| area | url | status | notes |
|---|---|---|---|
| docs-v5 root | https://www.okx.com/docs-v5/en/ | blocked | HTTP CONNECT 403 in this environment |
| docs-v5 root (alt) | https://my.okx.com/docs-v5/en/ | blocked | HTTP CONNECT 403 in this environment |
| changelog | https://www.okx.com/docs-v5/log_en/ | blocked | HTTP CONNECT 403 in this environment |
| upcoming changes | https://www.okx.com/docs-v5/en/#upcoming-changes | blocked | HTTP CONNECT 403 in this environment |
| rest auth | https://www.okx.com/docs-v5/en/#overview-rest-authentication | blocked | HTTP CONNECT 403 in this environment |
| ws overview | https://www.okx.com/docs-v5/en/#overview-websocket-overview | blocked | HTTP CONNECT 403 in this environment |
| errors | https://www.okx.com/docs-v5/en/#error-code | blocked | HTTP CONNECT 403 in this environment |
| rate limits | https://www.okx.com/docs-v5/en/#overview-rate-limit | blocked | HTTP CONNECT 403 in this environment |
| demo trading | https://www.okx.com/docs-v5/en/#overview-demo-trading-services | blocked | HTTP CONNECT 403 in this environment |

## Retrieval evidence
- command: `python - <<'PY' ... requests.get('https://www.okx.com/docs-v5/en/') ... PY`
- result: `ProxyError: Tunnel connection failed: 403 Forbidden`
