# EVM Adapter Policy

- Provider abstraction is mandatory (HTTP + optional WS).
- Every provider switch validates expected chain id.
- Signer is trait-based and secret material is never logged.
- Errors normalize to stable reason codes for SDK callers.
