# IR US Official Fetch Policy v1

Priority/fallback order:
1. JSON/API (if official/public/no-auth and available)
2. Official HTML detail fallback for filing context
3. Attachment discovery/download with metadata guard

Politeness requirements:
- user-agent/header discipline
- concurrency cap
- retry budget
- backoff policy
- attachment size guard
