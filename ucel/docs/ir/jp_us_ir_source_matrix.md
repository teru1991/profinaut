# JP/US IR Source Matrix

summary.sources.total: 7
summary.identities.total: 8
summary.documents.total: 16
summary.status.implemented: 2
summary.status.partial: 1
summary.status.not_implemented: 4

| market | source_family | source_id | source_kind | access_policy_class | status |
| --- | --- | --- | --- | --- | --- |
| jp | jp_statutory_disclosure | edinet_api_documents_v2 | official_public_api | free_public_noauth_review_required | implemented |
| jp | jp_timely_disclosure | jp_tdnet_timely_html | official_public_html | free_public_noauth_review_required | not_implemented |
| jp | jp_issuer_ir_site | jp_issuer_ir_html_public | issuer_ir_html | free_public_noauth_review_required | not_implemented |
| jp | jp_issuer_ir_site | jp_issuer_ir_feed_public | issuer_ir_feed | free_public_noauth_review_required | not_implemented |
| us | us_sec_disclosure | sec_edgar_submissions_api | official_public_api | free_public_noauth_allowed | implemented |
| us | us_issuer_ir_site | us_issuer_ir_html_public | issuer_ir_html | free_public_noauth_review_required | not_implemented |
| us | us_issuer_ir_site | us_issuer_ir_feed_public | issuer_ir_feed | free_public_noauth_review_required | partial |
