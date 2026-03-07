# IR US Official Identity Mapping v1

Required identity kinds for US official sources:
- `us_cik_like` (primary regulatory identity)
- `ticker_like` (lookup alias)
- `exchange_ticker_like` (secondary market alias)
- `url_like` (official filing URL locator)

Resolver requirements:
- preserve provenance per alias
- reject ambiguous matches by default
