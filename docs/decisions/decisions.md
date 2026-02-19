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
- 2026-02-19: EXCH-AUDIT-POSTB-000 により Task B（PR #156 merged）後の `services/marketdata` を監査し、UCELっぽい `descriptor/dsl/engine` 実装は `app/descriptor_dsl.py` に単一存在であることを確定 / Task B の成果を次タスクで再分岐させないため / `UCEL-CORE-POSTB-001` は既存 descriptor 実装を唯一の拡張先として進める。
- 2026-02-19: `gmo_adapter` 相当の別モジュールは存在せず、現行は `main.py`（REST poller）と `gmo_ws_connector.py`（WS ingest）の二導線 / 機能重複ではなく transport 分割のため即時削除は行わない / 次タスクでは operation SSOT を先に固定し、参照先を一本化した上で段階的に統合する。
- 2026-02-19: PR #140 系は close-without-merge 方針を維持し、post-TaskB 時点でも「取り込み不可（直接マージ禁止）」を継続 / 旧境界の再流入を防ぎ UCEL 実装を1本道に保つため / 必要差分は `UCEL-CORE-POSTB-001` 以降の新PRでのみ再実装する。
- 2026-02-19: 重複疑いとして `app/object_store.py` と `app/storage/object_store.py`、および `main.py` と `routes/health.py` の二重導線を記録し、このタスクでは additive-only で方針固定までとする / 破壊的削除を避けつつ次タスクの安全な集約順序を明確化するため / 参照先一本化→動作確認→不要側廃止を後続タスクで実施する。
- 2026-02-19: PROGRESS-AUDIT-000 で GitHub API 一次情報（open PR=0、#140=closed/no-merge、#156=merged、postB chain #172/#174/#175/#176/#177/#178/#179 merged）を Docs OS と突合し、次の唯一タスクを `POSTB-NEXT-001`（TASK C: GMO private execution hardening）に固定 / status/handoff/trace に残っていた「UCEL-CORE-POSTB-001 が次」という古い案内は GitHub 実態（既に関連PRが merged）と矛盾するため / 以後は PR #179 実装の運用証跡強化を最優先で進める。
