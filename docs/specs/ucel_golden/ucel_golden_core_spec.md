UCEL Golden Standard Core Spec（固定仕様書）v1.0
	•	Document ID: UCEL-GOLDEN-CORE-SPEC
	•	Status: Canonical / Fixed Contract
	•	Scope: 国内CEX（bitFlyer/bitbank/Coincheck）を基準にしつつ、仕様は venue に依存しない不変コアとして定義
	•	Non-goal: フェーズ、進捗、運用閾値、手順、保持日数、接続数などの「変動しうる情報」は含めない
	•	Compatibility: SemVer（後述）

⸻

0. 目的（不変の到達点）

本仕様は、Public WebSocket のデータ収集において、以下の不変要件（契約）を満たすことを目的とする。
	•	欠損ゼロを理想としつつ、現実の欠損を前提に「検知→回復→証明」を成立させる
	•	全取引所/全チャンネル/全通貨ペアを対象にスケール可能な枠組みを提供する
	•	「収集できている」主張は必ず証拠（audit/integrity/replay）で裏付けられる
	•	運用事故（誤操作/秘密漏洩/監視欠損）を前提に、常に安全側に倒れる

⸻

1. 不変要件（契約）

1-1. 欠損検知（No silent loss）
	•	データ欠損は「無かったこと」にしない
	•	欠損の疑い（gap/順序乱れ/遅延/重複）は必ず integrity に表面化する
	•	観測不能（監視欠損）は健康ではなく UNKNOWN として扱う

1-2. 再現性（Replayability）
	•	収集したデータは replay pointers により入力範囲が追跡可能である
	•	同一入力範囲を与えれば、少なくとも Evidence Replay（Type B）が成立する

1-3. 安全連動（Safety Interlock）
	•	重大な欠損/未知/監査不能は Safety Mode を SAFE に遷移させる
	•	SAFE/EMERGENCY_STOP 下では危険操作/実行系はブロックされる
	•	安全を緩める操作は必ず challenge/confirm を要求する

1-4. 監査可能性（Auditability）
	•	重要イベント（起動/停止/遷移/欠損/隔離/復旧/危険操作）は audit_event に残る
	•	監査ログに秘密値は含めない（secret-free）

1-5. SSOT分離（Core vs Policy vs Plan）
	•	Core Spec は契約固定であり、運用値は Policy に外出しする
	•	計画（進捗/段階/フェーズ）は Plan に外出しする
	•	Policy/Plan を変更しても Core の SemVer を変えない（契約不変）

⸻

2. カノニカル概念（固定語彙）

2-1. Safety Mode（固定）
	•	NORMAL / SAFE / EMERGENCY_STOP の3値が唯一の正

2-2. Evidence（固定）
	•	audit_event / integrity_report / gate_results / replay_pointers / support_bundle_manifest を証拠の基本セットとする

2-3. Quarantine（固定）
	•	異常は局所隔離（quarantine）できる
	•	隔離の入退は監査対象である

⸻

3. 互換性と SemVer（固定）

3-1. Core Spec SemVer
	•	MAJOR: 契約破壊（固定語彙/必達要件の変更）
	•	MINOR: 後方互換のある追加（新しい概念追加）
	•	PATCH: 誤字修正/説明補足（意味不変）

3-2. Policy/Plan
	•	運用値・計画は可変であり、Core Spec の SemVer を変えない
