# IR Source Taxonomy v1

## Scope
This taxonomy fixes JP/US IR source families for UCEL IR inventory.

## Source families
- `jp_statutory_disclosure`: official Japanese statutory disclosure source.
- `jp_timely_disclosure`: Japanese timely disclosure source.
- `jp_issuer_ir_site`: issuer-operated public IR pages/feeds in Japan.
- `us_sec_disclosure`: official SEC disclosure source in the US.
- `us_issuer_ir_site`: issuer-operated public IR pages/feeds in the US.

## source_kind enum
- `official_public_api`
- `official_public_feed`
- `official_public_html`
- `issuer_ir_html`
- `issuer_ir_feed`
- `attachment_download`

## access_pattern enum
- `api_list`
- `api_detail`
- `rss_poll`
- `feed_poll`
- `html_index`
- `html_detail`
- `attachment_discover`
- `artifact_download`
- `issuer_lookup`
- `search_query`
