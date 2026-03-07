# IR Issuer Sites Artifact Policy v1

Canonical artifact kinds for issuer sites:
- html, pdf, xbrl, ixbrl, xml, txt, csv, zip, json, rss

Fetch success requires metadata:
- content type
- encoding (if known)
- size
- checksum
- source URL
- referring page URL

Oversized/invalid content-type/unsupported kind must fail fast.
