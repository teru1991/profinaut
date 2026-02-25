UCEL Implementation Plan & Phase Playbook（計画書）v1.0
	•	Document ID: UCEL-IMPL-PLAN
	•	Status: Mutable / Execution-oriented
	•	Goal: 仕様（Core）を変えずに、「何をいつどこまでやるか」を管理する
	•	Rule: 計画は変わってよい。変わる前提。Core を揺らさない。

⸻

0. 管理単位（推奨）
	•	roadmap.md：全体ロードマップ
	•	phase1_public_ws.md：Phase1（Public WS）
	•	phase2_private.md：Phase2（Private）
	•	phase3_pipeline.md：Phase3（Silver/Gold/配信）
	•	venue_status.md：取引所別の進捗（SSOT整備、チャネル網羅、テスト状況）

⸻

1. Phase 1（Public WS）
	•	対象: 全取引所/全チャンネル/全通貨ペア
	•	必達: 欠損検知、再接続、再購読、証拠（audit/integrity/replay）
	•	DoD: “収集できている” を gate_results と integrity_report で証明できる

2. Phase 2（Private）
	•	対象: APIキー保護、private ws、アカウント/注文/約定の収集
	•	必達: secret管理、権限分離、監査

3. Phase 3（Pipeline）
	•	対象: Bronze/Silver/Gold、配信、再処理、バックフィル
	•	必達: append-only, lineage, replay, support bundle
