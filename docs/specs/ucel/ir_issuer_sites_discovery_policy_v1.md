# IR Issuer Sites Discovery Policy v1

Deterministic discovery order:
1. official source metadata seed (issuer URL/IR URL)
2. inventory fixed seed
3. deterministic root-site traversal (no search engine)

Forbidden:
- general search engine discovery
- ad-hoc unlimited crawl

Required outputs:
- issuer key binding with provenance
- candidate `IssuerSiteProfile` with section/feed/attachment rules
