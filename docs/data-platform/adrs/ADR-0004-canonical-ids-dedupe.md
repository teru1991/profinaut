# ADR-0004: Canonical IDs and dedupe/idempotency

- Status: Accepted
- Date: 2026-02-19

## Context
再送・重複・順序揺れがある market/private イベントで deterministic 処理が必要。

## Decision
- Bronze 全イベントに `canonical_id` と `idempotency_key` を必須化
- event type ごとの key 生成規則を storage-contracts.md で固定
- first-write-wins で重複排除

## Rationale
- 再計算時に同一結果を再現しやすい
- Silver/Gold の重複汚染を抑止

## Consequences
- adapter 実装に key 生成責務が追加
- DQ 指標に duplicate_ratio を必須追加
