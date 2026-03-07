# IR Identity Mapping Matrix

summary.identity_rows.total: 8

| source_id | market | identity_kind | canonical_role |
| --- | --- | --- | --- |
| edinet_api_documents_v2 | jp | jp_edinet_code_like | primary_regulatory_id |
| jp_tdnet_timely_html | jp | jp_exchange_code_like | secondary_market_id |
| jp_issuer_ir_html_public | jp | issuer_site_slug_like | issuer_site_locator |
| jp_issuer_ir_feed_public | jp | url_like | issuer_site_locator |
| sec_edgar_submissions_api | us | us_cik_like | primary_regulatory_id |
| sec_edgar_submissions_api | us | ticker_like | lookup_alias |
| us_issuer_ir_html_public | us | exchange_ticker_like | secondary_market_id |
| us_issuer_ir_feed_public | us | url_like | issuer_site_locator |
