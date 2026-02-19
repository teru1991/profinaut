# CHANGELOG (docs/exchanges/kraken)

## 2026-02-19
- Expanded `sources.md` to include explicit official URL evidence across Spot/Futures REST, Spot WS v1/v2, Futures WS, FIX, auth/errors/rate-limits, and changelog categories.
- Normalized WS/FIX IDs to the required naming format (including `ws.other` for Futures WS versioning and hyphenated session/message naming in FIX IDs).
- Rebuilt `catalog.json` so the machine-readable SSOT reflects the markdown endpoint/channel/feed catalog with required field metadata.
- Kept `diffs.md` as official-only placeholder with explicit environment limitation note because docs.kraken.com was inaccessible (proxy 403) during this run.
- Check date: 2026-02-19.
