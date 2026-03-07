# IR Access Guard Policy

Decision classes:
- `Allowed`
- `ReviewRequired`
- `Blocked`

Policy mapping:
- `FreePublicNoAuthAllowed` -> Allowed
- `FreePublicNoAuthReviewRequired` -> ReviewRequired
- `Excluded*` -> Blocked

Guard constraints:
- no paid/contract source
- no login-required flow
- no anti-bot bypass requirement
- API/feed/HTML/attachment are valid only when policy decision is non-blocked
