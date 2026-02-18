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
| `generator` | string | Yes | DSL source code for generating subscription messages (see DSL Grammar below) |
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
| `expressions` | [string] | No | — | Mini-expr expression strings (see Mini-Expr Engine below) |
| `max_expression_length` | usize | No | 4096 | Maximum allowed expression length |

---

## `[maps]` (optional)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol_map_file` | string | No | Path to symbol mapping file (relative to config dir) |
| `channel_map` | table | No | Channel name mapping (exchange → canonical) |

**Policy:** If `symbol_map_file` is specified but the file is not found, a warning is emitted (not a hard error).

---

## Subscription Generator DSL Grammar

The `generator` field in `[[subscriptions]]` contains DSL source code that
produces subscription messages. The DSL is parsed and executed at runtime
with the exchange instance's `symbols`, `channels`, and connection `id` as
context.

### Supported Statements

```
program     = statement*
statement   = foreach_stmt | if_stmt | emit_stmt
foreach_stmt = "foreach" "(" ident "in" collection ")" "{" statement* "}"
if_stmt     = "if" "(" expr ")" "{" statement* "}"
              ("else" "if" "(" expr ")" "{" statement* "}")*
              ("else" "{" statement* "}")?
emit_stmt   = "emit" "(" string_literal ")" ";"

collection  = "symbols" | "channels"
```

### Expression Grammar (conditions only)

```
expr     = or_expr
or_expr  = and_expr ( "||" and_expr )*
and_expr = comparison ( "&&" comparison )*
comparison = primary ( ("==" | "!=") primary )?
primary    = "(" expr ")" | string_literal | identifier
```

Allowed identifiers in conditions: loop variables (`symbol`, `ch`, etc.),
`conn_id`.

### String Literals

String literals use `"…"` or `'…'` delimiters. Supported escapes:
`\"`, `\'`, `\\`, `\n`.

### Line Comments

`// …` comments are supported and ignored.

### Example Generator

```
foreach(symbol in symbols) {
    foreach(ch in channels) {
        if (ch == "trades") {
            emit('{"method":"subscribe","channel":"trades","pair":"{symbol}"}');
        } else if (ch == "orderbook") {
            emit('{"method":"subscribe","channel":"book","pair":"{symbol}","depth":25}');
        }
    }
}
```

---

## Placeholder Substitution

Emitted strings from the DSL may contain placeholders that are substituted
at runtime. Placeholder syntax: `{name}`.

### Supported Placeholders

| Placeholder | Description |
|-------------|-------------|
| `{symbol}` | Current symbol value (from loop variable or context) |
| `{ch}` | Current channel value |
| `{channel}` | Alias for `{ch}` |
| `{conn_id}` | Connection ID bound to this generator |
| `{now_ms}` | Unix epoch milliseconds at evaluation time (per message) |
| `{uuid}` | UUID v4 generated fresh per occurrence |
| `{env:VAR}` | Environment variable `VAR` |
| `{arg:KEY}` | Argument from supplied context map |

### Policies

- **Unknown placeholder** → error (names the unknown placeholder)
- **Missing env var** → error (do not silently produce invalid auth/ping/subscription messages)
- **Missing arg** → error (names the missing key)
- **JSON content** → Literal `{` and `}` in JSON templates are safe; only
  `{identifier_chars}` patterns are treated as placeholders

---

## JSON Pointer Extraction (RFC 6901)

The `[parse]` section fields (`channel`, `symbol`, `server_time`, `sequence`,
`message_id`) are RFC 6901 JSON Pointer strings used to extract values from
incoming exchange messages.

### Pointer Syntax

- Must start with `/`
- Segments separated by `/`
- RFC 6901 escapes: `~0` → `~`, `~1` → `/`
- Array access by numeric index: `/items/0`

### Casting Rules

Extracted values can be cast to typed outputs:

| Target | Accepts | Numeric String Policy |
|--------|---------|----------------------|
| `u64` | JSON numbers, numeric strings | Accepted if fully numeric non-negative integer |
| `i64` | JSON numbers, numeric strings | Accepted if fully numeric integer |
| `string` | Strings, numbers, bools, null | All converted to string representation |
| `bool` | JSON bools, `"true"`/`"false"` strings | Only exact `"true"`/`"false"` accepted |

### Error Behavior

- Required field missing → error (includes pointer path)
- Optional field missing → `Ok(None)`
- Present but cast fails → error (includes value type and pointer path)
- `null` value treated as missing

---

## Mini-Expr Engine

When `parse.expr.enabled = true`, the mini-expr engine evaluates expressions
against incoming JSON payloads for flexible field extraction.

### Supported Features

| Feature | Syntax | Example |
|---------|--------|---------|
| Dot access | `a.b.c` | `data.price` |
| Array indexing | `a[N]` | `data.bids[0]` |
| Fallback | `x ?? y` | `ts ?? server_time` |
| `to_number()` | `to_number(expr)` | `to_number(data.price)` |
| `to_string()` | `to_string(expr)` | `to_string(data.seq)` |

### Strict Prohibitions

- No arithmetic (`+`, `-`, `*`, `/`)
- No user-defined functions
- No loops or recursion
- No external access (IO, network, env)

### Evaluation Rules

- Input is the JSON message payload
- Missing field → `null`
- `x ?? y`: if `x` is null → evaluate `y`, else `x`
- Array index out of range → `null`
- Unknown function → error
- Output is a `serde_json::Value`; cast using JSON Pointer casting rules

---

## Maps and Normalization

The optional `[maps]` section provides symbol and channel normalization.

### Symbol Map File

A TOML file with flat key-value pairs:

```toml
btcusdt = "BTC_USDT"
ethusdt = "ETH_USDT"
xrpusdt = "XRP_USDT"
```

### Channel Map (Inline)

Defined directly in the descriptor:

```toml
[maps.channel_map]
trade = "trades"
book = "orderbook_l2"
ticker = "ticker"
```

### Normalization Behavior

- `normalize_symbol(raw)` → mapped value if key exists, else `raw` (passthrough)
- `normalize_channel(raw)` → mapped value if key exists, else `raw` (passthrough)
- If `symbol_map_file` is configured but missing/unreadable → error

---

## Safety and Constraints

### DSL Execution Bounds

| Constraint | Default | Description |
|------------|---------|-------------|
| Max output messages | 1,000,000 | Per generator execution; error if exceeded |
| No recursion | — | DSL has no function definitions or recursive calls |
| Deterministic errors | — | Errors include line/column when possible |
| Scoped variables | — | Loop variables are scoped; inner loops can reference outer variables |

### Mini-Expr Bounds

| Constraint | Default | Description |
|------------|---------|-------------|
| Max expression length | 4,096 bytes | Configurable via `max_expression_length` |
| Max AST nodes | 1,000 | Prevents deeply nested expressions |
| Max evaluation steps | 10,000 | Prevents runaway evaluation |
| Function whitelist | `to_number`, `to_string` | Unknown functions are rejected |
| No external access | — | No IO, network, or environment variable access |

### Placeholder Safety

- Only `{identifier_chars}` patterns are treated as placeholders
- JSON braces in templates are safe (not misinterpreted as placeholders)
- Environment variables are read-only; missing vars produce errors
- Each `{uuid}` generates a fresh UUID v4
- Each `{now_ms}` reads the current clock (not cached)

---

## Typical Patterns

### Subscribe Template (Binance-style)

```
foreach(symbol in symbols) {
    foreach(ch in channels) {
        emit('{"method":"SUBSCRIBE","params":["{symbol}@{ch}"],"id":1}');
    }
}
```

### Auth + Subscribe (with env var)

```
emit('{"op":"auth","key":"{env:API_KEY}"}');
foreach(symbol in symbols) {
    emit('{"op":"subscribe","channel":"trades","instId":"{symbol}"}');
}
```

### Conditional Subscribe

```
foreach(ch in channels) {
    if (ch == "orderbook") {
        foreach(symbol in symbols) {
            emit('{"op":"subscribe","channel":"books","instId":"{symbol}","depth":"25"}');
        }
    } else {
        foreach(symbol in symbols) {
            emit('{"op":"subscribe","channel":"{ch}","instId":"{symbol}"}');
        }
    }
}
```

---

## Example Descriptor

See `config/crypto-collector/exchanges/example_v1_4.toml` for a complete
example that validates against the v1.4 schema.

## Task E Runtime Semantics (ACK / Failover / Signing)

### ACK gating

`subscriptions.ack` supports bounded ACK wait with optional correlation.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `field` | string | Yes | — | JSON pointer path evaluated on incoming payload |
| `value` | string | Yes | — | Expected value for ACK match |
| `correlation_pointer` | string | No | — | Optional pointer to correlate ACK per subscribe request |
| `timeout_ms` | u64 | No | 5000 | Timeout budget for bounded ACK wait |

Policy:
- If `correlation_pointer` is set, runtime waits until each generated subscribe request is acknowledged.
- On timeout, `subscribe_ack_timeout_total{exchange,connection}` increments and subscribe is retried with backoff.

### WS failover + reconnect

- Runtime reconnect uses exponential backoff with jitter.
- On repeated failures, URLs rotate through `ws.connections.urls` in order, cycling back to the first URL.
- Reconnect order is always: `connect -> auth (if any) -> subscribe -> ack gating -> running`.

### REST signing semantics

- REST secret material must be sourced from environment variables only.
- Descriptor/config may reference env key names, but must not embed secret literals.
- If required env secret is missing, instance enters `DEGRADED` with clear error and signed REST calls are not attempted.
