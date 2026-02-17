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

---

## Subscription Generator DSL (Task B)

The `generator` field in each subscription entry contains a DSL program that
produces subscription messages at runtime. The DSL is safe, bounded, and
deterministic.

### Grammar

```text
program     = statement*
statement   = foreach_stmt | if_stmt | emit_stmt

foreach_stmt = "foreach" "(" IDENT "in" IDENT ")" "{" statement* "}"
if_stmt      = "if" "(" condition ")" "{" statement* "}"
                ("else" "if" "(" condition ")" "{" statement* "}")*
                ("else" "{" statement* "}")?
emit_stmt    = "emit" "(" string_literal ")" ";"

condition   = or_expr
or_expr     = and_expr ("||" and_expr)*
and_expr    = cmp_expr ("&&" cmp_expr)*
cmp_expr    = "(" condition ")" | operand ("==" | "!=") operand
operand     = IDENT | string_literal
```

### Loop Collections

- `symbols` — the symbols array from the collector exchange instance config
- `channels` — the channels array from the collector exchange instance config

### Condition Identifiers

- `symbol` — current symbol loop variable
- `ch` / `channel` — current channel loop variable
- `conn_id` — connection ID for the subscription's binding

### Example

```text
foreach(symbol in symbols) {
    foreach(ch in channels) {
        if (ch == "trades") {
            emit("{channel}:{symbol}");
        } else {
            emit("book:{symbol}");
        }
    }
}
```

### Line Comments

Lines starting with `//` are ignored. Inline comments (`// ...`) are also supported.

---

## Placeholder Substitution (Task B)

Emitted strings support `{placeholder}` substitution. Use `{{` and `}}` for
literal brace characters.

| Placeholder | Description |
|-------------|-------------|
| `{symbol}` | Current symbol value from loop |
| `{ch}` | Current channel value from loop |
| `{channel}` | Alias for `{ch}` |
| `{conn_id}` | Generator-bound connection ID |
| `{now_ms}` | Unix epoch milliseconds at evaluation time |
| `{uuid}` | UUID v4 (unique per occurrence) |
| `{env:VAR}` | Environment variable `VAR` |
| `{arg:KEY}` | Context argument `KEY` |

**Policies:**

- Unknown placeholders produce an error (not silently ignored).
- Missing environment variables produce an error (do not silently produce
  invalid subscription messages).
- `{{` produces a literal `{`; `}}` produces a literal `}`.

---

## Mini-Expression Evaluator (Task B)

When `parse.expr.enabled = true`, expression strings from
`parse.expr.expressions` are evaluated against the JSON message payload.

### Supported Syntax

```text
expr        = fallback_expr
fallback    = access ("??" access)*
access      = primary (("." IDENT) | ("[" NUMBER "]"))*
primary     = IDENT | function_call | string_literal | number_literal
function    = IDENT "(" expr ")"
```

### Features

| Feature | Example | Description |
|---------|---------|-------------|
| Dot access | `data.symbol` | Access nested JSON fields |
| Array index | `data.trades[0]` | Index into JSON arrays |
| Fallback | `data.sym ?? data.symbol ?? "unknown"` | First non-null value |
| `to_number(x)` | `to_number(data.price)` | Cast string/bool to number |
| `to_string(x)` | `to_string(data.seq)` | Cast number/bool to string |

### Behavior

- Missing field → `null`
- Array index out of range → `null`
- `x ?? y` → if `x` is null, evaluate `y`
- Unknown function → error

### Strict Prohibitions

- No arithmetic (`+`, `-`, `*`, `/`)
- No user-defined functions
- No loops or recursion
- No external access (IO, network, environment variables)

---

## Safety / Constraints

The DSL, placeholder engine, and expression evaluator enforce strict bounds
to prevent resource exhaustion and ensure deterministic behavior.

| Constraint | Default | Description |
|------------|---------|-------------|
| Max DSL output messages | 1,000,000 | Per generator execution |
| Max expression length | 4,096 bytes | Per expression string |
| Max AST nodes (expr) | 1,000 | Per expression parse tree |
| Max evaluation steps (expr) | 10,000 | Per expression evaluation |
| Placeholder nesting | None | No nested or recursive templates |

Error messages include:
- **DSL:** line/column for syntax errors; subscription index + connection ID for runtime errors
- **Placeholders:** placeholder name for unknown/missing errors
- **Pointers:** pointer path and value type for extraction/cast errors
- **Expressions:** position for parse errors; function name for unknown functions
