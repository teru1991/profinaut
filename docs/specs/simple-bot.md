# simple_mm spec

- Path: `bots/simple_mm/main.py`
- Goal: shortest E2E verification for `ticker -> paper order intent -> order/fills log`
- Safety invariant: in degraded or SAFE_MODE, no new order is submitted
  - If controlplane is unreachable, block new orders
  - If controlplane capabilities is `status=degraded`, block new orders
- Logging: lines include `run_id`, `bot_id`, `state`, `decision` (and `order_id` on order result)
- One-command run:

```bash
python bots/simple_mm/main.py
```
