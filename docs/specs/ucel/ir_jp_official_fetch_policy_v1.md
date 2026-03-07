# IR JP Official Fetch Policy v1

Priority/fallback order:
1. API (if official/public/no-auth and available)
2. Official feed polling for discovery
3. Official HTML index/detail fallback
4. Attachment discovery/download with metadata guard

Politeness requirements:
- concurrency cap
- retry budget
- backoff policy
- attachment size guard
