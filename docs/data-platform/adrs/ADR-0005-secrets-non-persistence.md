# ADR-0005: Secrets non-persistence

- Status: Accepted
- Date: 2026-02-19

## Context
データ基盤に credential/token/signature を残すと漏えい面積が増える。

## Decision
- secrets は Bronze/Silver/Gold/Serving いずれにも永続化しない
- denylist + regex scrub + schema gate を必須化
- ログも同一ポリシーを適用

## Rationale
- 事故時の被害範囲最小化
- 監査で説明可能な一貫ルール

## Consequences
- collector 前段で scrub が必須
- 例外/障害時の dump でも秘匿情報は残らない設計にする
