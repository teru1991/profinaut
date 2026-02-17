# Exchange Descriptor Schema v1.4 — Reference

This document describes the exchange descriptor format used by the
Multi-Exchange Market Data Collector Framework v1.4 (Crypto Subsystem).

Each exchange has one descriptor file (TOML) that defines how to connect,
subscribe, and parse messages from that exchange.

---

## Top-Level Sections

| Section | Required | Description |
|---------|----------|-------------|
| `meta` | Yes | Exchange name and schema version |
| `ws` | Yes | WebSocket connection definitions |
| `rest` | No | REST API configuration |
| `subscriptions` | Yes | Subscription generators per connection |
| `parse` | Yes | JSON pointer paths for message parsing |
| `maps` | No | Symbol and channel mapping tables |

---

## `[meta]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Human-readable exchange name (must match config instance expectation) |
| `version` | string | Yes | Descriptor schema version (should be `"1.4"`) |

---

## `[[ws.connections]]`

Each entry defines a WebSocket connection endpoint.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | — | Unique connection identifier (referenced by subscriptions) |
| `urls` | [string] | Yes | — | WebSocket URLs (non-empty; first is primary, rest are fallbacks) |
| `tls` | table | No | — | TLS settings |
| `read_timeout_ms` | u64 | No | 30000 | Read timeout in milliseconds |
| `keepalive` | table | No | — | Keepalive/ping configuration |

### `[ws.connections.tls]`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | bool | No | true | Enable TLS |
| `ca_cert_path` | string | No | — | Custom CA certificate path |

### `[ws.connections.keepalive]`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `mode` | string | Yes | — | Keepalive mode: `"ping_frame"`, `"application"`, `"none"` |
| `interval_ms` | u64 | No | 30000 | Keepalive interval in milliseconds |
| `template` | string | No | — | Application-level ping message template (used when mode is `"application"`) |

**Validation:** Connection `id` values must be unique. `urls` must be non-empty.

---

## `[rest]` (optional)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `base_urls` | [string] | Yes (if section present) | REST API base URLs (non-empty) |
| `rate_limit` | table | No | Rate limiting configuration |

### `[rest.rate_limit]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `requests_per_minute` | u32 | No | Simple rate limit |
| `token_bucket` | table | No | Token bucket rate limiter |

### `[rest.rate_limit.token_bucket]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `capacity` | u32 | Yes | Bucket capacity |
| `refill_per_second` | f64 | Yes | Refill rate |

---

## `[[subscriptions]]`

Each entry defines a subscription sent over a WebSocket connection.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `connection_id` | string | Yes | Must reference an existing `ws.connections.id` |
| `generator` | string | Yes | DSL expression for generating subscription messages (non-empty; NOT executed in Task A) |
| `ack` | table | No | Acknowledgement matcher |

### `[subscriptions.ack]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | string | Yes | JSON pointer to the ack field |
| `value` | string | Yes | Expected value |

---

## `[parse]`

JSON pointer paths for extracting fields from exchange messages.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `channel` | string | Yes | Pointer to channel identifier |
| `symbol` | string | Yes | Pointer to symbol/pair |
| `server_time` | string | No | Pointer to server timestamp |
| `sequence` | string | No | Pointer to sequence number |
| `message_id` | string | No | Pointer to message ID |
| `expr` | table | No | Expression engine settings |

All pointer strings must follow RFC 6901 format (start with `/`).

### `[parse.expr]` (optional)

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | bool | No | false | Enable expression evaluation |
| `expressions` | [string] | No | — | Expression strings (validated for length only in Task A) |
| `max_expression_length` | usize | No | 4096 | Maximum allowed expression length |

---

## `[maps]` (optional)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol_map_file` | string | No | Path to symbol mapping file (relative to config dir) |
| `channel_map` | table | No | Channel name mapping (exchange → canonical) |

**Policy:** If `symbol_map_file` is specified but the file is not found, a warning is emitted (not a hard error).

---

## Example Descriptor

See `config/crypto-collector/exchanges/example_v1_4.toml` for a complete
example that validates against the v1.4 schema.
