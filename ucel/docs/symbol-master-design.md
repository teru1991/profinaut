# UCEL Symbol Master / Registry Design SSOT

This document defines the UCEL-side lib-only components for symbol normalization:

- `ucel-symbol-core`: common instrument model, decimal policy, snapshot schema.
- `ucel-symbol-adapter`: fetch/subscribe traits, connector capabilities, rate-limit policy.
- `ucel-symbol-store`: in-memory SSOT + diff engine + monotonic `store_version`.

The runtime loop, persistence operation, scheduling, and metrics exposure are service responsibilities and are implemented under `services/marketdata-rs/symbol-master`.
