# Collector Config (`collector.toml`) — Reference

This document describes the collector configuration file used by the
Multi-Exchange Market Data Collector Framework v1.4 (Crypto Subsystem).

The collector config defines runtime settings and the list of exchange
instances to manage.

---

## Top-Level Sections

| Section | Required | Description |
|---------|----------|-------------|
| `[run]` | Yes | Runtime settings (port, logging) |
| `[[exchange]]` | Yes (at least one) | Exchange instance definitions |

---

## `[run]`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `http_port` | u16 | Yes | — | HTTP server port for `/healthz` (must be > 0) |
| `log_level` | string | No | `"info"` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

---

## `[[exchange]]`

Each entry defines a managed exchange instance.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | — | Unique instance name |
| `enabled` | bool | No | true | Whether to activate this instance |
| `descriptor_path` | string | Yes | — | Path to the exchange descriptor TOML (relative to config file) |
| `symbols` | [string] | Yes | — | Symbols to subscribe to |
| `channels` | [string] | Yes | — | Channels to subscribe to |
| `overrides` | table | No | — | Instance-level overrides (parsed but reserved for future use) |

### Validation Rules

1. **`http_port`** must be in range 1–65535.
2. **Exchange `name`** values must be unique across all instances.
3. **Enabled instances** must have non-empty `symbols` and `channels`.
4. **Disabled instances** (`enabled = false`) are validated for shape but
   their descriptor files are not loaded.
5. **`descriptor_path`** must be non-empty. File existence is checked at
   service startup (not during config-only validation).
6. Error messages include the exchange instance name for context.

---

## Example

```toml
[run]
http_port = 8090
log_level = "info"

[[exchange]]
name = "binance"
enabled = true
descriptor_path = "exchanges/binance_v1_4.toml"
symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT"]
channels = ["trades", "orderbook_l2"]

[[exchange]]
name = "kraken"
enabled = true
descriptor_path = "exchanges/kraken_v1_4.toml"
symbols = ["XBT/USD", "ETH/USD"]
channels = ["trades", "book"]

[exchange.overrides]
read_timeout_ms = 15000

[[exchange]]
name = "gmo"
enabled = false
descriptor_path = "exchanges/gmo_v1_4.toml"
symbols = []
channels = []
# Disabled — descriptor will not be loaded.
```

See `config/crypto-collector/collector.toml` for the shipped sample
configuration.
