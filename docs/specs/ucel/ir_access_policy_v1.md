# IR Access Policy v1

## Allowed classes
- `free_public_noauth_allowed`
- `free_public_noauth_review_required`

## Excluded classes
- `excluded_paid_or_contract`
- `excluded_login_required`
- `excluded_policy_blocked`

## Rule
Targets in this series must be free/public/no-auth, and must not require CAPTCHA or anti-bot bypass.
API/feed/HTML/attachment patterns are all valid if policy class is allowed.
