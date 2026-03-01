# Runbook: WS rate-limit nack → penalty/cooldown

## Symptoms
- rl_penalty_applied / rl_cooldown_set が増える
- 特定 op_id / key が繰り返し pending→inflight を繰り返す
- /healthz が Degraded（RL_COOLDOWN_HIGH 等）

## What to check
- metrics: rl_penalty_applied, rl_cooldown_set, deadletter_count
- events_tail: "ws rate-limit -> applied limiter penalty" / cooldown set
- support_bundle: rules_snapshot / events_tail / metrics

## Immediate actions
1) rules: stability.buckets.public_rps/private_rps を下げる
2) rules: stability.rate_limit.default_penalty_ms を増やす
3) max_attempts で落ちた deadletter を確認し、原因（取引所仕様変更等）を切り分け

## Recovery
- penalty/cooldown が収束する
- deadletter が増え続けない
