# Equity Adapter Policy

- Equity price-data adapters are separate from IR/disclosure adapters.
- Vendors must expose explicit latency class metadata (realtime/delayed/end_of_day).
- Missing exchange code/timezone/session data is treated as partial support or error.
- Canonical models are the only SDK-facing surface; raw vendor payload is adapter-internal.
