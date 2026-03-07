# US Official Access and Politeness

Decision classes:
- Allowed
- ReviewRequired
- Blocked

Rules:
- free_public_noauth_allowed => Allowed
- free_public_noauth_review_required => ReviewRequired (explicit approval required)
- excluded classes => Blocked

Politeness policy keys:
- user_agent
- concurrency_cap
- retry_budget
- base_backoff_ms
- max_attachment_bytes

Guard behavior:
- enforce header discipline (`User-Agent`)
- enforce retry/backoff path
- enforce attachment size cap
- reject blocked/review-required flows by default
