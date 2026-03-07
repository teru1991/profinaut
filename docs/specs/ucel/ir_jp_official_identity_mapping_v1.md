# IR JP Official Identity Mapping v1

Required identity classes:
- `jp_edinet_code_like`
- `jp_local_code_like`
- `jp_exchange_code_like`
- name/title-like alias via provenance
- URL-like source alias

Rules:
- canonical issuer key is market-scoped
- aliases are source-scoped
- provenance is mandatory
- ambiguous resolution must fail or review_required
