# Decisions Log

Format: `YYYY-MM-DD: Decision / Rationale / Consequences`

- 2026-02-17: 1PR=1scope / Limits blast radius and simplifies review / Work should remain narrowly scoped per PR.
- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
- 2026-02-17: docs canonical入口 is `docs/SSOT/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
