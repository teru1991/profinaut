# WS Ingest Policy

- Supervisor owns retry/resume/deadletter decisions for public/private ingest.
- Runtime hooks report typed failures; venue ad-hoc reconnect logic should be minimized.
- Retry budget is per stream; backoff budget is per venue/family.
- Stall detection is mandatory even if ACK succeeded.
