# Runbook: WS reconnect storm / circuit breaker open

## Symptoms
- reconnect が短時間に連続（storm guard 近傍/発火）
- breaker が Open のまま
- /healthz が Degraded（RECONNECT_FAILURE / BREAKER_OPEN）

## What to check
- metrics: reconnect_attempts, reconnect_failure, breaker_open, last_inbound_age_ms
- events_tail: reconnect_reason, breaker_state

## Immediate actions (safe)
1) 対象 venue の WS を一時停止（subscriber単位）
2) rules の stability.buckets.public_rps/private_rps を下げる（負荷低減）
3) breaker.cooldown_ms を上げる（再試行を間引く）

## Root cause hints
- 取引所側障害 / ネットワーク / DNS
- こちらの subscribe burst（mps過大）
- idle timeout が短すぎる

## Recovery
- 取引所復旧後に breaker が HalfOpen→Closed に遷移することを確認
- 収束しない場合は spill/overflow や RL も同時に確認
