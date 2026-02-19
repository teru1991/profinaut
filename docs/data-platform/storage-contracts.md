# Storage Contracts (Bronze/Silver/Gold)

## 1. Bronze record contract (JSON Schema)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://profinaut.local/schemas/bronze-record.schema.json",
  "title": "BronzeRecord",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "event_type",
    "source",
    "source_event_id",
    "canonical_id",
    "idempotency_key",
    "event_time",
    "ingested_at",
    "payload",
    "meta"
  ],
  "properties": {
    "event_type": {
      "type": "string",
      "enum": [
        "trade",
        "orderbook_delta",
        "orderbook_snapshot",
        "ticker",
        "execution",
        "balance",
        "funding_rate",
        "mark_price"
      ]
    },
    "source": {
      "type": "object",
      "required": ["exchange", "channel", "transport"],
      "properties": {
        "exchange": { "type": "string" },
        "channel": { "type": "string" },
        "transport": { "type": "string", "enum": ["ws", "rest", "fix", "grpc", "file"] }
      },
      "additionalProperties": false
    },
    "source_event_id": { "type": "string", "minLength": 1 },
    "canonical_id": { "type": "string", "minLength": 1 },
    "idempotency_key": { "type": "string", "minLength": 1 },
    "event_time": { "type": "string", "format": "date-time" },
    "ingested_at": { "type": "string", "format": "date-time" },
    "payload": { "type": "object" },
    "meta": {
      "type": "object",
      "required": ["schema_version", "collector", "raw_ref"],
      "properties": {
        "schema_version": { "type": "string" },
        "collector": { "type": "string" },
        "raw_ref": { "type": "string", "minLength": 1 },
        "transform_version": { "type": "string" }
      },
      "additionalProperties": true
    }
  }
}
```

## 2. Secret scrub denylist（永続化禁止）
以下に一致する key/value は Bronze/Silver/Gold いずれにも保存しない。

- Header keys: `authorization`, `proxy-authorization`, `x-api-key`, `x-auth-token`, `cookie`, `set-cookie`
- Query/body keys: `api_key`, `apikey`, `secret`, `client_secret`, `access_token`, `refresh_token`, `signature`, `sig`, `passphrase`, `private_key`
- Exchange-specific variants: `apiKey`, `apiSecret`, `recvWindowSignature`, `X-BAPI-SIGN`, `X-MBX-APIKEY`
- Credential-like values（regex）:
  - `(?i)bearer\s+[a-z0-9\-\._~\+\/]+=*`
  - `(?i)aws(.{0,20})?(secret|token)`
  - `(?i)(ed25519|rsa)?_?private_?key`

実装要件:
- ingest 前に scrub filter を適用
- schema gate で denylist key を reject
- raw payload dump 時にマスキング（`***REDACTED***`）

## 3. Dedupe / idempotency key policy per event type
- `trade`:
  - `idempotency_key = {exchange}:{symbol}:{trade_id}`
- `orderbook_delta`:
  - `idempotency_key = {exchange}:{symbol}:{sequence}`
- `orderbook_snapshot`:
  - `idempotency_key = {exchange}:{symbol}:{snapshot_ts}`
- `ticker`:
  - `idempotency_key = {exchange}:{symbol}:{event_time}`
- `execution`:
  - `idempotency_key = {exchange}:{account}:{order_id}:{fill_id}`
- `balance`:
  - `idempotency_key = {exchange}:{account}:{asset}:{event_time}`
- `funding_rate`:
  - `idempotency_key = {exchange}:{symbol}:{funding_time}`
- `mark_price`:
  - `idempotency_key = {exchange}:{symbol}:{event_time}`

同一 `idempotency_key` は first-write-wins。

## 4. RawRef spec (Silver/Gold -> Bronze tracing)
`raw_ref` は Silver/Gold レコードから Bronze の原本へ辿るための URI。

### Format
```text
raw://bronze/{exchange}/{event_type}/dt={YYYY-MM-DD}/hh={HH}/part-{file_id}#{source_event_id}
```

### Requirements
- Silver row は `raw_ref` を必須保持
- Gold aggregate は `raw_refs`（配列）または `lineage_ref` を保持
- lineage は 365日以上保持（監査要件）

### Example
```text
raw://bronze/binance/trade/dt=2026-02-19/hh=10/part-000187.parquet#834920119
```
