# Decisions Log

Format: `YYYY-MM-DD: Decision / Rationale / Consequences`

- 2026-02-17: 1PR=1scope / Limits blast radius and simplifies review / Work should remain narrowly scoped per PR.
- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
- 2026-02-17: docs canonical入口 is `docs/SSOT/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
- 2026-02-17: DOC-FIX-001 bootstrap completed / preflight docs missing/invalid / Created/repaired docs OS preflight SSOT files to unblock task generation and lock-safe execution.
- 2026-02-18: DOC-FIX-GMO-000 preflight gate enforced / GMO final implementation tasks require lock-state certainty and traceable SSOT references / Block task issuance until LOCK conflicts are resolved and status open_prs are refreshed.
- 2026-02-19: PR #140 は merge せず close し、新しい exchange-library 基盤（EXCH-LIB-CORE 系）へ必要差分を再統合する / #140 は旧前提での safety-boundary 実装であり、現行の再設計方針と差分管理（lock/trace/責務分離）を崩すリスクがあるため / 既存PRの差分は参照専用にし、採用が必要な要素は新基盤前提の新PRで取り込む。
- 2026-02-19: 例外条件として #140 をそのまま採用する場合は「base branch を最新 master に rebase 済み」「EXCH-LIB-CORE 境界との責務整合レビュー完了」「required locks/trace-index/status/handoff の更新完了」「回帰テスト証跡が再取得済み」を全て満たすこと / 条件未達でのマージは SSOT 不整合を招くため禁止 / 条件を満たせない場合は close-without-merge を維持する。
- 2026-02-19: PR #140 close-without-merge の決定を実行済み（Closed on 2026-02-19） / UCEL v1.1.4 の統一基盤へ寄せるため / 二重境界と再設計コストの増大を回避。
