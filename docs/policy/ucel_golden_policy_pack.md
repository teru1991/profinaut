UCEL Golden Policy Pack（運用ポリシー集）v1.0
	•	Document ID: UCEL-GOLDEN-POLICY
	•	Status: Mutable / Environment-bound
	•	Goal: Core Spec の契約を破らずに、運用値（しきい値・保持・上限・通知）を調整可能にする
	•	Rule: 本ファイルの変更は Core Spec の SemVer を変えない（契約は不変、値だけ変える）

⸻

0. 運用ファイル一覧（推奨）
	•	thresholds.toml：SLO/閾値
	•	forbidden_keys.toml：秘密漏洩ガード
	•	retention.toml：保持/圧縮/退避
	•	remote_access.toml：Access/Grafana など運用値
	•	hardware_policy.toml：UPS/SSD/冗長など（任意）

⸻

1. thresholds.toml（例）
	•	latency.ingest_p95_ms = 500
	•	latency.execution_p95_ms = 2000
	•	backlog.storage_seconds_warn = 10
	•	backlog.storage_seconds_fail = 30
	•	gap.max_time_gap_ms_warn = 1000
	•	gap.max_time_gap_ms_fail = 5000
	•	observability.targets_missing_grace_seconds = 60

⸻

2. forbidden_keys.toml（例）
	•	api_key
	•	secret
	•	token
	•	private_key
	•	authorization
	•	cookie

⸻

3. retention.toml（例）
	•	raw_retention_days = 30
	•	canonical_retention_days = 180
	•	derived_retention_days = 365
	•	compaction_schedule = "weekly"

⸻

4. remote_access.toml（例）
	•	cloudflared_enabled = true
	•	grafana_public = false
	•	allowlist_ips = []

⸻

5. safety_interlock（例）
	•	integrity_fail_to_safe = true
	•	gate_unknown_to_safe = true
	•	replay_pointer_missing_to_safe = true

⸻

6. support_bundle（例）
	•	trigger_on_safe = true
	•	trigger_on_emergency_stop = true
	•	trigger_on_integrity_fail = true
	•	max_bundles_per_day = 5
