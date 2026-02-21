# Binance EXD-004 Auth Strict Verification Notes

- confirmed_date: 2026-02-21
- primary refs:
  - https://developers.binance.com/docs/binance-spot-api-docs/rest-api/request-security
  - https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/request-security
- supporting official mirror (Binance official GitHub docs):
  - https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md
  - https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/web-socket-api.md

## Verified decisions
1. Auth type:
   - REST SIGNED endpoints: signature required (HMAC/RSA/Ed25519 key types supported by official docs).
   - WS API SIGNED methods: signature required in `params`.
2. Required headers/fields:
   - REST signed: `X-MBX-APIKEY` header + `timestamp` + `signature` (+ optional `recvWindow`).
   - WS signed: `params.apiKey`, `params.timestamp`, `params.signature` (+ optional `recvWindow`).
3. Timestamp/recvWindow:
   - timestamp accepted in ms or μs; recvWindow unit is milliseconds (decimal up to 3 places), default 5000, max 60000.
4. Signature base string:
   - REST: query string + body (concatenated without separator) after percent-encoding non-ASCII.
   - WS: all request params except signature, sorted alphabetically by parameter name, then `k=v` joined by `&`.
5. Canonicalization:
   - REST body canonicalization is `application/x-www-form-urlencoded` payload semantics (not JSON canonicalization).
   - WS signs UTF-8 bytes of canonical string.

## Cross-check
- Official sample payload HMAC recomputed and matched official sample digest in docs.
- command output file: `exd-004-hmac-check.log`
