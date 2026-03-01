# Runbook: WS Overflow Spill

1. Confirm outq_dropped/outq_spilled counters.
2. Check overflow mode and spill_dir free space.
3. If spill is disabled, consider temporary slow-down policy.
4. Verify consumer throughput and WAL queue lag.
5. Collect support bundle and include event tail around overflow.
