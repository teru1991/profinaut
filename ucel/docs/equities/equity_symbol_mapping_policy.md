# Equity Symbol Mapping Policy

- Canonical key = market + exchange_code + symbol.
- Vendor aliases/suffixes are normalized into canonical symbol records.
- Ambiguous symbol mapping fails closed (no silent fallback).
- Mapping drift is detected by fixture-backed tests.
