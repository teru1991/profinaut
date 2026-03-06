# Public Adapter Policy

- Public-only venues are **supported** when ticker/trades/orderbook/symbols are available.
- Private surface availability is independent from public market-data support.
- Venue raw payload must be normalized into UCEL canonical market-data models before SDK exposure.
- WS behavior must define ack mode, heartbeat behavior, reconnect/resubscribe behavior, and integrity mode.
- checksum/gap support may be partial by venue and must be documented in the matrix.
