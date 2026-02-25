# FX / Macro Data Ingestion & Normalization Core Spec v1.0（固定仕様）
FX rates / Macro indicators / Calendar events / Revision-aware facts / Evidence replay

- Document ID: FX-MACRO-INGEST-NORM-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): P（FX / Macro）
- Depends-on（Fixed）:
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - データソース優先順位、更新頻度、許容遅延、保持期間、丸め/スプレッド処理、改定の採用方針 → `docs/policy/**`
  - 障害対応/復旧/切替/問い合わせ手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
FX・マクロは「更新頻度が異なる」「遅延がある」「後から改定される」「複数ソースで値がズレる」を前提に扱う必要がある。
本仕様は、FX/Macroデータを **欠損・遅延・改定・不一致**を前提に取り込み、
それでも **真実を隠さず、再現・証明可能**な形で正規化し、下流（リスク/戦略/レポート）に安全に供給する基盤を固定する。

必達要件（固定）：
1) **Raw-first evidence**：取得した原データ（応答/配信）を証拠として保持できる（または安定参照）
2) **Revision-aware facts**：後から改定される値は上書きせず、改定イベントとして表現する
3) **Normalization boundary**：raw（証拠）/facts（正規化）/derived（派生）を分離する
4) **No silent loss**：欠損・不明・監視欠損は integrity_report に必ず出す
5) **Multi-source integrity**：ソース間不一致は UNKNOWN/DEGRADED として表面化（隠さない）
6) **Replayable**：replay_pointers で「どの入力・どの版・どのルールで使ったか」を追える
7) **Safety honesty**：監視欠損・整合性不明は SAFE 側へ（crosscut準拠）
8) **No secrets**：APIキー等は secret_ref のみ（平文禁止）

---

## 1. 範囲（in / out）
### 1.1 In
- FXレート（スポット/参考レート/ブローカー配信等）の取得と正規化
- マクロ指標（CPI/GDP/雇用統計等）の取得と正規化
- 経済カレンダー（発表時刻/予想/結果）の取得と正規化
- 改定（revision）モデル（速報→改定値）
- Multi-sourceクロスチェック（不一致検出）
- Integrity/Gate inputs（欠損/遅延/改定/不一致）
- replay pointers / audit events

### 1.2 Out
- 特定プロバイダ固有の実装詳細（adapter/policyへ）
- 売買ロジック/戦略
- UI詳細（ただし説明可能性要件は固定）

---

## 2. Core Concepts（固定）
### 2.1 Instrument Identity（FX）
FXの対象は最低限以下で識別できる：
- `base_ccy` / `quote_ccy`（例：USD/JPY）
- `pair`（例：USDJPY、ただし表記は正規化）
- `venue_or_source`（参照元：broker/provider）
- `rate_type`（mid/bid/ask/reference 等）
- `stream_id`（上記を安定に連結）

### 2.2 Macro Series Identity（マクロ）
マクロ時系列は最低限以下で識別できる：
- `country`
- `series_id`（例：CPI, GDP, NFP 等の正規化ID）
- `frequency`（tick/daily/monthly/quarterly）
- `unit`（%、index、currency 等）
- `seasonal_adjustment`（sa/nsa等：可能なら）
- `source`
- `series_key`（上記を安定に連結）

### 2.3 Event Identity（カレンダー）
発表イベントは最低限以下を持つ：
- `event_id`（内部一意）
- `series_key`
- `scheduled_at_utc`
- `release_time_precision`（exact/approx/unknown）
- `source`
- `version`（改定・差替えにより増える）
- `quality`（OK/DEGRADED/UNKNOWN）

---

## 3. Layers（固定：Raw / Facts / Derived）
### 3.1 Raw（証拠）
- API応答、配信フレーム、CSV等（形式は自由）
- content hash（改ざん検知）
- 取得メタ（取得時刻UTC、source、request params要約）
固定ルール：
- Raw欠落は再現性欠落として integrity に影響
- Secret（token/cookie等）は保存しない（redaction必須）

### 3.2 Facts（正規化）
- FX: timestamp付きレート（bid/ask/mid）、スプレッド、quality
- Macro: (period, value, unit, vintage/version, quality)
- Calendar: (scheduled, actual, forecast, previous, vintage/version, quality)

### 3.3 Derived（派生）
- リターン、ボラ、サプライズ（actual-forecast）、指標スコア等
固定ルール：
- Derivedは再計算可能であること（inputsとpolicy snapshot refで再現）

---

## 4. Time Semantics（固定）
### 4.1 UTC統一（固定）
- すべての時刻はUTCで扱える
- ローカル時刻は表示用途に限る

### 4.2 Time precision（固定）
- 発表時刻は “精度” を持つ（exact/approx/unknown）
- precisionが低いイベントを高精度として扱わない（qualityへ反映）

---

## 5. Revision / Vintage（改定）モデル（固定）
### 5.1 原則（固定）
- 値の改定は起きる前提
- 上書き禁止（append-only）
- 改定は “新しい vintage/version” と “restate event” で表現する

### 5.2 Restate event（固定要件）
改定が発生したら、少なくとも以下を記録できる：
- `restate_id`
- `series_key` / `event_id`
- `from_version` → `to_version`
- `reason`（sourceの改定/修正/差替え等）
- `diff_summary`（何が変わった）
- `evidence_refs`（raw hashes/keys）
- `actor`（auto/manual）

---

## 6. Multi-source Integrity（固定）
### 6.1 不一致の扱い（固定）
- 同一時刻・同一系列で、複数ソースが異なる値を返すことがある
固定ルール：
- 不一致は “平均して丸める” などで隠蔽しない
- `quality=DEGRADED/UNKNOWN` を許容し、integrityへ反映する
- どの値を採用したか（primary source）と、採用理由（policy snapshot）を記録できる

### 6.2 採用戦略（固定枠組み）
- primary/secondary を Policy で定義できる
- primary欠損時に secondary へフェイルオーバーできる
- ただしフェイルオーバーは audit と integrity に記録される（静かに切替しない）

---

## 7. FX特有：bid/ask/mid とスプレッド（固定）
- bid/ask がある場合、midは派生として算出できる
- bid/ask欠損時は mid の quality を下げる（DEGRADED/UNKNOWN）
- スプレッド異常は anomaly として integrity に影響（閾値はPolicy）

---

## 8. Macro/Calendar特有：予想・結果・前回（固定）
- forecast/actual/previous は別フィールドとして保持
- previous の改定（前回値の修正）を restatement として扱える
- “速報値” と “確報値” は version/vintage で区別できる

---

## 9. Integrity / Gate inputs（固定）
最低限、以下の integrity signals を生成できる：
- expected updates vs observed（系列/通貨ペア単位）
- fetch failures / parse failures
- missing intervals（時刻欠損）
- staleness（最新更新からの遅れ）
- revision counts（改定頻度）
- source disagreement rate（不一致率）
- observability missing intervals（監視欠損）

固定ルール：
- 監視欠損は UNKNOWN
- UNKNOWN は crosscut safety により SAFE 側へ（最低限）

---

## 10. Replay pointers（固定）
replay_pointers は少なくとも参照できる：
- series/pairs set + window
- raw evidence keys（保存参照とhash）
- facts window（パーティション/キー）
- derived outputs（必要なら）
- policy snapshot ref（採用戦略/丸め/フィルタ）
- binary_hash / config_hash（可能な範囲）

---

## 11. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `fxmacro.fetch.start/end`（window/source/targets）
- `fxmacro.raw.stored`（hash + key）
- `fxmacro.parse.fail`（error.kind + ref）
- `fxmacro.facts.upserted`（counts + quality distribution）
- `fxmacro.restatement.applied`（from→to + reason + refs）
- `fxmacro.source.failover`（primary→secondary）
- `integrity.record` / `gate.record`
- `support_bundle.created`（必要時）

---

## 12. Safety coupling（固定）
- integrity FAIL/UNKNOWN（大量欠損/監視欠損/大規模不一致）時は SAFE 側へ
- UNKNOWNのデータを “確定” として使うことを防ぐ（下流がqualityを参照できること）

---

## 13. テスト/検証観点（DoD）
最低限これが検証できること：

1) raw evidence が保存/参照され、hashで同一性が検証できる
2) 改定が上書きでなく restatement として表現される
3) 欠損/不一致が integrity_report に必ず出る（silent loss無し）
4) 同じ inputs_ref+policy_snapshot_ref で同じ派生成果、または差分説明が残る
5) 監視欠損で gate UNKNOWN → safety SAFE 側へ寄る（crosscut）
6) failover が静かに起きず、audit/integrityに残る

---

## 14. Policy/Runbookへ逃がす点
- ソース優先順位、更新頻度、許容遅延、保持期間、丸め/採用戦略、閾値
- 障害対応/切替/復旧/問い合わせ手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
