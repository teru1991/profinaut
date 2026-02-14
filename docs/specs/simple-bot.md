# simple_mm spec

- Path: `bots/simple_mm/main.py`
- Goal: shortest E2E verification for `ticker -> paper order intent -> order/fills log`
- Safety invariant: in degraded or SAFE_MODE, no new order is submitted
- Logging: each line includes `run_id` and `bot_id`
- One-command run:

```bash
python bots/simple_mm/main.py
```
