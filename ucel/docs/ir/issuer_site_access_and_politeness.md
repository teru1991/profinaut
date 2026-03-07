# Issuer Site Access and Politeness

Decisions:
- Allowed
- ReviewRequired
- Blocked

Policy rules:
- free/public/no-auth only
- review_required requires explicit approval (default deny)
- blocked classes fail fast

Politeness controls:
- concurrency_cap
- retry_budget
- crawl_depth_cap
- page_budget
- base_backoff_ms
- max_attachment_bytes
