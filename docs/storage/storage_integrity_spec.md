# Storage / Persistence / Integrity Core Spec v1.0（固定仕様）
Raw-first Data Lake / DB Persistence / WAL / Integrity Evidence

- Document ID: STOR-INTEGRITY-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): E（Storage / Data Lake / Persistence）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Market Data Collector: `docs/specs/market_data/collector_framework_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - retention、圧縮、パーティション粒度、SLO、再試行上限、容量閾値 → `docs/policy/**`
  - 復旧・移行・リカバリ手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、収集データと実行データを **安全・堅牢・高速・安定**に永続化し、
欠損・重複・遅延・部分故障があっても **真実を隠さず、再現・証明可能**にする不変条件を固定する。

必達要件（固定）：
1) **Raw-first is canonical**：外部入力の事実（RawFrame）は再現の正本として保持できる
2) **No silent data loss**：失われた/不明な区間は integrity_report で必ず表面化する
3) **Deterministic references**：replay_pointers で入力範囲・設定・出力を追跡できる
4) **Tamper-evidence**：改ざんがあれば検出できる（hash/manifest/append-onlyの設計）
5) **WAL safety**：永続の安全性が崩れる兆候は crosscut safety と連動（SAFEへ）
6) **Isolation**：環境（dev/stage/prod、paper/shadow/live）を交差させない
7) **Performance**：高頻度でもバックプレッシャで破綻せず、縮退は制御される

---

## 1. 責務境界（in / out）
### 1.1 In（対象）
- Raw-first データレイク（append-only の入力証拠）
- Canonical（正規化イベント）の永続（DB/TSDB等は実装自由）
- WAL/コミット境界（少なくとも概念として）
- 取り込み遅延・欠損・重複の可視化（integrity signals）
- replay_pointers の永続参照
- スキーマ互換と schema_version の保持

### 1.2 Out（対象外）
- 具体DB製品の最適化ノウハウ（別ドキュメントで扱える）
- BI/可視化
- 戦略実装

---

## 2. データ階層（固定概念：Raw → Canonical → Derived）
### 2.1 Raw（正本）
- 外部から受け取った “事実” を可能な限り未加工で保存
- append-only
- 受信順序は保証しない（事実の保存に徹する）

### 2.2 Canonical（利用層）
- Raw から正規化されたイベント（UCEL型など）
- dedupe/ordering/snapshot sync の結果として安定化される
- Raw と突合できる参照（raw_ref / payload_hash 等）を持てること（固定要求）

### 2.3 Derived（派生層）
- 集計、特徴量、インデックス等
- 失敗しても Raw/Cannonical の正本性を損なわない

---

## 3. Append-only / Commit boundary（固定）
### 3.1 Append-only の不変条件
- Raw は原則追記のみ（削除・更新は禁止）
- 例外（retention/compaction）は Policy + Runbook + Audit でのみ実施
- 破壊的操作は dangerous op として扱い、challenge/confirm + audit 必須

### 3.2 Commit boundary（固定概念）
永続の安全は “コミット境界” を持つ：
- Raw ingest の「受信→永続」までが確認できる
- Canonical の「生成→永続」までが確認できる
- 境界の欠損/不明は Integrity の FAIL/UNKNOWN 要因

---

## 4. Schema / Versioning（固定）
### 4.1 schema_version の固定
- 出力する構造化データは schema_version を必ず持つ（契約SSOTに準拠）
- schema_version が不明なデータは “利用禁止” 扱い（隔離）

### 4.2 互換性（固定）
- schema変更は versioning_policy に従い、破壊変更は新schema_versionへ
- 古いデータは “読める/再現できる” ことが前提（replay）

---

## 5. Integrity signals（固定）
Storageは次を最低限生成・保持できること：

- raw_ingest_count / bytes
- canonical_persist_count
- backlog / lag（受信→永続の遅延）
- gaps（時間ギャップ、シーケンスギャップの検知結果）
- dedupe_suppressed（重複抑止数）
- missing_intervals（監視欠損・保存欠損）
- quarantine impacts（影響範囲）
- checksum/manifest results（改ざん検知）

これらは integrity_report の根拠になる。

---

## 6. integrity_report（固定：真実表明）
integrity_report は “運用の真実” を表明する固定物。
- 期間（window）を持つ
- PASS/WARN/FAIL/UNKNOWN を持つ（契約SSOTに準拠）
- “Unknown は安全ではない” として crosscut safety と連動する

最低限記載できること（固定要求）：
- 対象範囲（venues/streams）
- expected vs observed
- gaps / missing_intervals
- raw→canonical の遅延
- quarantine summary
- evidence refs（replay pointers 等）

---

## 7. replay_pointers（固定）
replay_pointers は “再現のための地図”。
最低限、次を参照できること：
- Raw の範囲（object keys/partitions/time window）
- Canonical の範囲（テーブル/パーティション/時刻）
- 使用した SSOT/plan/config の hash
- 生成物（integrity_report/gate_results）の ref

audit_event から参照されること（固定）。

---

## 8. Safety interlock 連携（固定）
以下は storage hazard として扱い、crosscut safety に連動：

- WAL/persist failure
- disk near-full / IO stall
- integrity_report FAIL
- observability missing intervals（真実不明）

固定ルール：
- hazard が検出されれば System Safety Mode は少なくとも SAFE
- 破壊的データ操作は SAFE/EMERGENCY_STOP で拒否（例外なし）

---

## 9. Isolation（固定）
- dev/stage/prod のデータは交差しない（物理/論理いずれでも）
- paper/shadow/live は最低でも名前空間・パーティションが分離される
- 交差が疑われた場合は Integrity FAIL 扱い

---

## 10. Performance / Backpressure（固定）
- 書き込みが詰まったら backpressure をかける（silent drop禁止）
- 収集側は backlog を観測し、必要なら P2 を縮退（Policy）
- “縮退した事実” は必ず integrity_report に出る

---

## 11. Audit（固定）
最低限、以下を audit_event として残す：
- integrity.record（integrity_report_ref）
- replay_pointers.record（replay_pointers_ref）
- storage.backpressure（開始/終了、影響範囲）
- retention/compaction（dangerous opとして challenge/confirm + 実行結果）
- restore/repair（dangerous op）

秘密値は絶対に含めない。

---

## 12. テスト/検証観点（DoD）
最低限これが検証できること：

1) Raw が append-only である（破壊操作は challenge/confirm 無しにできない）
2) silent drop しない（欠損は integrity に必ず出る）
3) replay_pointers が生成され、入力範囲が追える
4) integrity_report が FAIL/UNKNOWN のとき safety が SAFE 側へ寄る
5) env/mode のデータが交差しない
6) disk/IO hazard で破壊操作が拒否される

---

## 13. Policy/Runbookへ逃がす点
- retention/圧縮/パーティション粒度/容量閾値
- restore/repair 手順
- SLO（許容遅延・許容欠損）
→ Policy/Runbookへ（意味は変えない）

---
End of document
