# IR Access Policy (Series 014)

## Allowed
- `free_public_noauth_allowed`
- `free_public_noauth_review_required`

## Excluded
- `excluded_paid_or_contract`
- `excluded_login_required`
- `excluded_policy_blocked`

## Allowed access patterns
- api_list, api_detail
- rss_poll, feed_poll
- html_index, html_detail
- attachment_discover, artifact_download
- issuer_lookup, search_query

Policy scope includes API/feed/HTML/attachment retrieval as long as source is free/public/no-auth.
