# Exchange Verification Report: binance

## 0. Metadata
- Exchange: binance
- Verification scope (files):
  - `docs/exchanges/binance/README.md`
  - `docs/exchanges/binance/sources.md`
  - `docs/exchanges/binance/rest-api.md`
  - `docs/exchanges/binance/websocket.md`
- Reviewer: Codex
- Status: `調査中`
- Last updated: 2026-02-21

## 1. Official Sources (Primary)
- Official site entry point: https://www.binance.com/en
- Developer docs entry path: Binance site (`/en`) → Developers portal (`developers.binance.com`) → Spot API Docs
- Final API doc URL(s): https://developers.binance.com/docs/binance-spot-api-docs/README
- Auth doc URL: https://developers.binance.com/docs/binance-spot-api-docs/rest-api/request-security
- Changelog URL: https://developers.binance.com/docs/binance-spot-api-docs/changelog
- Confirmed date: 2026-02-21

### Evidence
- Navigation/URL memo: `docs/exchanges/_verification/evidence/binance/exd-002-official-source-note.md`
- HTTP reachability logs (environment-limited):
  - `docs/exchanges/_verification/evidence/binance/exd-002-curl-binance-site.log`
  - `docs/exchanges/_verification/evidence/binance/exd-002-curl-developers.log`
  - `docs/exchanges/_verification/evidence/binance/exd-002-curl-spot-readme.log`

## 2. Coverage (EXD-003)
- REST Public: Yes
- REST Private: Yes (partial)
- WebSocket Public: Yes
- WebSocket Private: Yes
- Rate limit documented: Yes (partial)
- Error model documented: Yes (partial)

### 2.1 Gaps
- Missing APIs:
  - Spot REST で `amend` / `batch` の対応可否の明示が不足（対応する場合のエンドポイント、非対応なら明示）。
  - Transfer領域（deposit/withdraw/address）の整理不足（Spot API対象か、別製品APIか、NAか）。
- Unnecessary APIs:
  - 現時点では未検出（No）。
- Ambiguous points:
  - Spot現物における positions/leverage/margin を `No` ではなく `NA` として明文化する必要。
  - Rate limit（weight, order count, IP制限, ban条件, backoff推奨）の実装指針が分散。
  - Error model（HTTPコード/独自コード/リトライ可否）が実装単位で正規化されていない。

### Evidence
- `docs/exchanges/_verification/evidence/binance/exd-003-coverage-matrix.md`

## 3. Auth Details (EXD-004)
- Auth type: SIGNED (HMAC / RSA / Ed25519) for secured methods
- Required headers:
  - REST: `X-MBX-APIKEY`
  - WS API: `params.apiKey`（headerではなくparams）
- Timestamp unit & tolerance:
  - `timestamp`: milliseconds or microseconds
  - `recvWindow`: milliseconds only, default 5000, max 60000, up to 3 decimal places
- Signature base string:
  - REST: query string + request body (no separator), non-ASCIIはpercent-encode後に署名
  - WS API: `signature`以外のparamsをキー名アルファベット順で `k=v` 連結（`&`区切り）
- Query/body canonicalization:
  - REST signed payloadは `application/x-www-form-urlencoded` 形式（JSON canonicalization 前提ではない）
  - WS API は canonical文字列の UTF-8 bytes を署名
- Notes vs official sample:
  - REST HMAC sample payload の署名を OpenSSL で再計算し、公式サンプルと一致を確認。
  - 署名生成の疑義: 現時点で検出なし（0件）。

### Evidence
- `docs/exchanges/_verification/evidence/binance/exd-004-auth-verification-notes.md`
- `docs/exchanges/_verification/evidence/binance/exd-004-rest-request-security-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-004-ws-request-security-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-004-hmac-check.log`

## 4. P0 Endpoints (EXD-005)
- Ticker:
  - Endpoint: `GET /api/v3/ticker/price`
  - Params: `symbol`(opt), `symbols`(opt)
  - 条件付き必須/排他: `symbol` と `symbols` は同時指定不可（同時指定時 `-1102`）
  - Response: 単一シンボル時 `object` / 複数時 `array<object>`
- Orderbook:
  - Endpoint: `GET /api/v3/depth`
  - Params: `symbol`(req), `limit`(opt; default 100, max 5000)
  - Response: `lastUpdateId`, `bids[]`, `asks[]`（`[price, qty]` string配列）
- Place order:
  - Endpoint: `POST /api/v3/order` (SIGNED)
  - Common required: `symbol`, `side`, `type`, `timestamp`
  - 条件付き必須:
    - `LIMIT`: `timeInForce`, `quantity`, `price`
    - `MARKET`: `quantity` or `quoteOrderQty`
    - `STOP_LOSS`/`TAKE_PROFIT`: `quantity` + (`stopPrice` or `trailingDelta`)
    - `STOP_LOSS_LIMIT`/`TAKE_PROFIT_LIMIT`: `timeInForce`, `quantity`, `price`, (`stopPrice` or `trailingDelta`)
- Cancel order:
  - Endpoint: `DELETE /api/v3/order` (SIGNED)
  - Required: `symbol`, `timestamp`
  - 条件付き必須: `orderId` OR `origClientOrderId`
- Balance:
  - Endpoint: `GET /api/v3/account` (SIGNED)
  - Required: `timestamp`
  - Optional: `recvWindow`, `omitZeroBalances`
  - Response key fields: `balances[].asset`, `balances[].free`, `balances[].locked`

### Evidence
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-validation-notes.md`
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-ticker-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-depth-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-new-order-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-cancel-order-excerpt.md`
- `docs/exchanges/_verification/evidence/binance/exd-005-p0-account-excerpt.md`

## 5. Smoke Test Evidence (EXD-006)
- Pending

## 6. Fixes Applied (EXD-007)
- Pending

## 7. Continuous Follow-up (EXD-008)
- Pending
