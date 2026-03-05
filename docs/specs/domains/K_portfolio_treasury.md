# K: Portfolio / Treasury (Core Ledger Contract)

## Scope (K-LEDGER-CORE-001)
This document fixes the minimal core to operate K ledger safely:
- Event-sourced ledger (append-only)
- Tamper-evident hash chain
- SBOR (state builder) replay determinism
- Valuation (marks/fx) + Confidence
- PnL (FIFO lot + basic attribution)
- Explain (lineage + confidence reasons)

## Ledger event schema (versioned)
All events MUST include:
- schema_version: int
- event_id: str (uuid)
- ts_utc: ISO8601 string
- source: str
- account: str
- kind: enum
- refs: { tx_ref?: str, venue?: str, symbol?: str }

Kinds (core):
- DEPOSIT / WITHDRAW
- TRANSFER
- TRADE_FILL
- FEE
- FUNDING
- INTEREST
- PRICE_MARK
- FX_RATE
- ADJUSTMENT (explicit correction; past events are never rewritten)

## Tamper-evident
- Stored rows: (seq, ts_utc, prev_hash, payload_json, record_hash)
- record_hash = SHA256(prev_hash + "\n" + payload_json)
- Any mismatch => ledger is invalid (fail-close for downstream).

## SBOR determinism
- Replaying the same event sequence must yield identical state (byte-wise stable JSON where applicable).
- Duplicate event_id must be ignored (idempotent apply).

## Valuation & Confidence
- If required marks/fx are missing, confidence decreases and reasons are reported.
- Valuation must never invent prices; missing => unknown with explicit reason.

## PnL (core)
- FIFO lots per (account, asset)
- Realized vs Unrealized separated
- Fee/Funding/Interest attributed as separate components

## Explain
- Must provide:
  - positions / cash summary
  - lineage references (event_ids) contributing to current state
  - confidence score (0..1) and reasons
  - pnl breakdown components and refs
