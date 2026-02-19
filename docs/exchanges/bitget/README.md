# Bitget Official Docs Catalog (SSOT)

This directory contains an **official-docs-only** catalog for Bitget:
- REST: Spot / Margin / Mix(Futures) / Broker / Copy / Earn / UTA (as documented)
- WebSocket: channels + protocol details (as documented)
- Official diffs from Changelog / Release notes / Deprecation notices

Completeness is evidenced by `sources.md`.

> Note: During this run, direct access to `www.bitget.com` returned HTTP 403 via the execution proxy, so endpoint/channel level extraction could not be completed from source pages in this environment.

## Files
- sources.md: SSOT evidence of coverage
- rest-api.md: REST endpoints catalog (by domain + public/private)
- websocket.md: WebSocket catalog (by domain + public/private + common)
- fix.md: FIX catalog (only if officially documented; else not_applicable w/ evidence)
- data.md: Official data distribution catalog (if documented)
- diffs.md: Official API diffs (changelog/release/support; official only)
- catalog.json: Machine-readable SSOT (1:1 with markdown tables)
- templates.md: Update templates
- CHANGELOG.md: Append-only change history
