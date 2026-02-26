# Level 1 SSOT Outline（F: Time / Clock Discipline）

## 0. Metadata
- Domain: **F**
- Title: **Time / Clock Discipline — 実装目標機能詳細設計 v1.4（最終版）**
- Source: F.txt
- Versioning: time schema / contract changes are treated as **SemVer contract** and gated by CI（詳細は本文参照）

---

## 1. Scope and Non-negotiables

### 1.1 Purpose
- システム全体の時刻を **観測可能・説明可能・再現可能** にする。
- 時計の乱れ（NTP不調、OS時刻ジャンプ、leap/step、取引所タイムスタンプ異常、分散環境skew、攻撃/改変、仕様変更、起動直後不安定）を前提に **安全に縮退** し、監査・復旧・再現（replay）まで破綻させない。

### 1.2 Non-negotiable Requirements（必達）
1. 全イベントは **recv_time（wall+mono）・clock_state_id・host_id** を必ず持つ  
2. **event_time / recv_time / persist_time** を統一定義し、**time_quality** と **reason** を付与  
3. NTP/OS/コンテナ/プロバイダ（将来PTP含む）健全性監視、異常時 **DEGRADED/UNSAFE** へ遷移  
4. venue（取引所）ごとの **offset と uncertainty** を推定し、補正根拠を監査可能に残す  
5. wall が step/leap/逆行しても、mono基準で timeout/遅延/レート制限/安全性を維持  
6. 低品質時刻データは「保存はするが、意思決定・執行には入れない」を契約化  
7. 分散（複数ホスト）でホスト間skewの観測・相関・replayを可能にする  
8. 時刻仕様・設定・プロファイル変更は **契約（SemVer）** として扱い、CIで回帰検知  
9. 攻撃/改変を脅威として扱い、最小限の防御と監査を備える  
10. 後からの補正・再正規化（バックフィル）を可能にし、当時値の再現性を壊さない  
11. 起動直後・障害時の運用容易化：証跡エクスポート・注釈・環境差プロファイル・定期レポート  

---

## 2. Canonical Model（SSOT）

### 2.1 Clock Domains
- wall_time: UTC
- mono_time: monotonic

### 2.2 Unified Time Definitions（3時刻）
- event_time: 発生源時刻
- recv_time: 受信時刻（必須：wall+mono）
- persist_time: 永続化完了時刻（該当する場合必須）

### 2.3 Timezone / DST
- 内部: UTC ns 固定
- 外部入力: RFC3339 + TZ 必須

---

## 3. Canonical Contract（All-event Common Fields）

### 3.1 Timestamp Fields（ns, UTC）
**必須**
- recv_utc_ns: i64  
- recv_mono_ns: i64  
- clock_state_id: ClockStateId  
- host_id: HostId  
- time_quality: TimeQuality  
- time_quality_reason: ReasonCode  
- payload_hash: Hash256  
- time_signature: Hash256  
- time_schema_version: TimeSchemaVersion  

**任意**
- event_utc_ns: Option<i64>
- persist_utc_ns: Option<i64>
- venue_time_offset_ns: Option<i64>
- venue_time_uncertainty_ns: Option<i64>
- event_time_adjusted: bool
- event_time_source: {Exchange, LocalDerived}
- normalized_event_utc_ns: Option<i64>

**拡張観測点**
- recv_socket_ns: Option<i64>
- recv_app_ns: Option<i64>

**起動直後ガード用**
- startup_phase: enum {Warmup, Normal}（イベントに付与可能） **(F-58)**

### 3.2 Invariants
- mono逆行 → E_INVALID
- persist < recv → E_INVALID
- 未来/逆行閾値超え → D_SUSPECT

---

## 4. State Models

### 4.1 ClockState（グローバル時計状態）
- （v1.3同等）＋起動フェーズ・環境プロファイル参照（F-58 / F-60）
- 追加:
  - startup_phase: {Warmup, Normal}
  - environment_profile_id: EnvProfileId

### 4.2 VenueClockState（取引所補正）
- （v1.3同等）

### 4.3 TimeQuality / ReasonCode
- （v1.3同等）＋追加 reason:
  - StartupWarmup
  - EnvProfileRestricted
  - EvidenceExported
  - OperatorAnnotationPresent

---

## 5. Services / Interfaces

### 5.1 TimeService
- （v1.3同等）＋追加:
  - export_time_evidence(window, filters) -> EvidenceBundleRef **(F-57)**
  - attach_annotation(range, note, tags) -> AnnotationId **(F-59)**

### 5.2 Provider / Reference Consensus
- （v1.3同等）

---

## 6. Behavior / Policies（Canonical）

### 6.1 Drift / Step / Slew
- （v1.3同等）

### 6.2 Venue Calibrator + Guard
- （v1.3同等）

### 6.3 Normalization / Ordering / Sanity
- （v1.3同等）

### 6.4 Degrade Policy
- （v1.3同等）

### 6.5 Time Budget / Quality Policy / Profiles / Drift
- （v1.3同等）

### 6.6 Backfill / Schema Version / Governance / CI Gate
- （v1.3同等）

---

## 7. Additional Capabilities（F v1.4 additions）

### 7.1 Evidence Bundle（証跡パッケージ） **(F-57)**
- 目的: 障害時の調査・再現・監査を高速化するため、直近X分の時刻関連情報を一括エクスポート
- 含める内容（固定）:
  - ClockStateEvent（期間内）
  - VenueClockStateEvent（期間内）
  - /timez スナップショット（開始/終了）
  - 主要メトリクス要約（p50/p95/max、違反回数）
  - 代表ログ断片（clock/venue/budget/schema-drift/tls）
  - 設定スナップショット（time.toml / env profile / venue profile参照ID）
  - 署名検証結果の要約（mismatch有無）
- 出力仕様:
  - “アーカイブ参照ID”（EvidenceBundleRef）を返す（保存先は実装依存）
  - 監査ログに EvidenceExported を記録

### 7.2 Warmup Control（コールドスタート最適化） **(F-58)**
- WarmupGracePeriod:
  - 起動後 WARMUP_GRACE の間は startup_phase=Warmup
  - Warmup中の規則（安全側）:
    - A_EXCHANGE_EVENT_CONFIRMED を原則出さない（最低B）
    - venue offset は Warmup 扱い
    - Execution への time_ok は（設定で）抑制できる
- 終了条件（例）:
  - ntp_sync==Synced が一定継続 + uncertainty <= U2 + venue最小サンプル達成 など

### 7.3 Operator Annotations（運用注釈） **(F-59)**
- 目的: 人間側イベント（回線切替、OS更新等）を紐付け、原因究明を補助
- 仕様:
  - Annotation { time_range, note, tags, created_by, created_at }
  - /timez やダッシュボードで表示可能
  - イベント側に OperatorAnnotationPresent を理由として付与可能

### 7.4 Env Time Profile（環境差プロファイル） **(F-60)**
- 目的: dev/stg/prod、Windows/Mac差などのノイズ吸収、アラート汚染防止
- EnvProfile（例）:
  - probe_capabilities（取れる指標）
  - threshold_overrides（許容値）
  - alert_suppression_rules（抑制）
  - startup_warmup_overrides
- 運用:
  - environment_profile_id を ClockState に保持
  - profile変更は Governance/CI 対象（v1.3ルール）

### 7.5 External Read-only Health（外部公開ヘルス） **(F-61)**
- 目的: 外出先ダッシュボード等が安全に参照できる read-only endpoint を正式化
- 仕様:
  - /timez を read-only で公開できる設計
  - 返すのは要約（詳細は内部のみ）:
    - clock_health, time_ok, uncertainty, venue_regime_summary, budget_violations
  - 認可は他ドメインの認証基盤に従う（F側は前提のみ）

### 7.6 Health Report（週次/月次の健康診断レポート） **(F-62)**
- 目的: 劣化予兆検出と運用改善の定量化
- 指標（例）:
  - drift推移（週平均/最大）
  - step/leap回数
  - ntp_unsynced時間
  - venue suspect回数・uncertainty推移
  - budget違反回数
  - schema drift検出回数
  - quarantine件数
  - tls failure相関（時計起因っぽい割合）
- 出力:
  - 自動生成（ジョブ/cronは実装側）
  - EvidenceBundle と相互リンク可能

---

## 8. Observability（Metrics / Storage）

### 8.1 Additional Metrics（v1.4）
- startup_warmup_seconds_total
- evidence_export_total
- operator_annotations_total
- env_profile_active{profile}
- time_health_report_generated_total

### 8.2 Storage
- TODO: 証跡アーカイブ保存先・保持期間・アクセス制御（EvidenceBundleRef の実体）  
  （本文では「保存先は実装依存」とのみ記載）

---

## 9. Tests / DoD（Behavior / Verification）

### 9.1 Additional Tests（v1.4）
- Warmup期間中の品質抑制
- EvidenceBundleの内容整合
- EnvProfileでの閾値上書きとアラート抑制
- 注釈の紐付け表示
- レポート生成の集計検証

### 9.2 Definition of Done（v1.4）
- v1.3 の DoD に加え:
  - Evidence export ができる
  - Warmup制御があり、起動直後事故が起きにくい
  - EnvProfileで環境差を吸収できる
  - 注釈が残せる
  - 週次/月次の健康診断が出せる（少なくとも出せる設計と出力フォーマットが確定）

---

## 10. Capability Index（IDs）
- **F-57**: Evidence Bundle（証跡パッケージ） / export_time_evidence -> EvidenceBundleRef
- **F-58**: Warmup制御 / startup_phase（Warmup/Normal） / WarmupGracePeriod
- **F-59**: Operator Annotations / attach_annotation -> AnnotationId / OperatorAnnotationPresent
- **F-60**: Env Time Profile / environment_profile_id / threshold_overrides / suppression
- **F-61**: 外部公開（Read-only）ヘルス / /timez read-only summary
- **F-62**: 週次/月次の健康診断レポート
# Level 2 Deep Spec（F: Time / Clock Discipline v1.4）

> NOTE: 新しい仕様は追加せず、入力の整理のみ。v1.3同等と書かれている範囲は **TODO** として未展開（本文に定義が無いため）。

## 1. Canonical Time Model

### 1.1 Clock Domains
- wall_time: UTC
- mono_time: monotonic

### 1.2 Canonical Timestamps
- event_time: 発生源時刻
- recv_time: 受信時刻（wall+mono 必須）
- persist_time: 永続化完了時刻（該当時必須）

### 1.3 Timezone Handling
- internal: UTC ns 固定
- external input: RFC3339+TZ 必須

---

## 2. Contract: Event Timestamp Envelope

### 2.1 Required Fields
- recv_utc_ns: i64
- recv_mono_ns: i64
- clock_state_id: ClockStateId
- host_id: HostId
- time_quality: TimeQuality
- time_quality_reason: ReasonCode
- payload_hash: Hash256
- time_signature: Hash256
- time_schema_version: TimeSchemaVersion

### 2.2 Optional Fields
- event_utc_ns: Option<i64>
- persist_utc_ns: Option<i64>
- venue_time_offset_ns: Option<i64>
- venue_time_uncertainty_ns: Option<i64>
- event_time_adjusted: bool
- event_time_source: {Exchange, LocalDerived}
- normalized_event_utc_ns: Option<i64>

### 2.3 Optional Observability Hooks
- recv_socket_ns: Option<i64>
- recv_app_ns: Option<i64>

### 2.4 Startup Guard Attachment (F-58)
- startup_phase: enum {Warmup, Normal}（イベントに付与可能）

### 2.5 Invariants / Validation
- mono逆行 → E_INVALID
- persist < recv → E_INVALID
- 未来/逆行閾値超え → D_SUSPECT

---

## 3. State: ClockState / VenueClockState

### 3.1 ClockState（Global）
- Base: v1.3同等（未展開）
- Additions:
  - startup_phase: {Warmup, Normal}
  - environment_profile_id: EnvProfileId
- TODO: v1.3の ClockState スキーマ定義（本入力に存在しない）

### 3.2 VenueClockState（Per-venue）
- Base: v1.3同等（未展開）
- TODO: v1.3の VenueClockState スキーマ定義（本入力に存在しない）

---

## 4. TimeQuality / ReasonCode

### 4.1 Base
- v1.3同等（未展開）
- TODO: v1.3の TimeQuality レベル定義（本入力に存在しない）

### 4.2 Additional Reason Codes (v1.4)
- StartupWarmup
- EnvProfileRestricted
- EvidenceExported
- OperatorAnnotationPresent

---

## 5. Services

### 5.1 TimeService API Additions
- export_time_evidence(window, filters) -> EvidenceBundleRef **(F-57)**
- attach_annotation(range, note, tags) -> AnnotationId **(F-59)**
- TODO: v1.3の TimeService API 一覧（本入力に存在しない）

---

## 6. Evidence Bundle (F-57)

### 6.1 Objective
- 直近X分の時刻関連情報を一括エクスポートし、調査・再現・監査を高速化

### 6.2 Bundle Contents (Fixed)
- ClockStateEvent（期間内）
- VenueClockStateEvent（期間内）
- /timez スナップショット（開始/終了）
- 主要メトリクス要約（p50/p95/max、違反回数）
- 代表ログ断片（clock/venue/budget/schema-drift/tls）
- 設定スナップショット（time.toml / env profile / venue profile参照ID）
- 署名検証結果の要約（mismatch有無）

### 6.3 Output and Audit
- EvidenceBundleRef を返す（保存先は実装依存）
- 監査ログに EvidenceExported を記録
- TODO: 保存先（ストレージ種別・暗号化・保持期間・アクセス制御）

---

## 7. Warmup Control (F-58)

### 7.1 WarmupGracePeriod
- 起動後 WARMUP_GRACE の間は startup_phase=Warmup

### 7.2 Warmup Rules (Safety-first)
- A_EXCHANGE_EVENT_CONFIRMED を原則出さない（最低B）
- venue offset は Warmup 扱い
- Execution への time_ok は（設定で）抑制できる

### 7.3 Exit Conditions (Examples)
- ntp_sync==Synced が一定継続 + uncertainty <= U2 + venue最小サンプル達成 など
- TODO: 具体的な閾値（WARMUP_GRACE, U2, サンプル条件）

---

## 8. Operator Annotations (F-59)

### 8.1 Annotation Schema
- Annotation { time_range, note, tags, created_by, created_at }

### 8.2 Surfacing and Tagging
- /timez やダッシュボードで表示可能
- OperatorAnnotationPresent を reason としてイベントに付与可能
- TODO: tags の語彙・制約、created_by の主体（ユーザ/ロール）と監査

---

## 9. Env Time Profile (F-60)

### 9.1 EnvProfile Components (Examples)
- probe_capabilities
- threshold_overrides
- alert_suppression_rules
- startup_warmup_overrides

### 9.2 Operational Rules
- environment_profile_id を ClockState に保持
- profile変更は Governance/CI 対象（v1.3ルール）
- TODO: v1.3の Governance/CI 具体ルール（本入力に存在しない）

---

## 10. External Read-only Health (F-61)

### 10.1 Endpoint
- /timez を read-only で公開できる設計

### 10.2 Returned Summary Fields
- clock_health
- time_ok
- uncertainty
- venue_regime_summary
- budget_violations

### 10.3 AuthZ
- 認可は他ドメインの認証基盤に従う（F側は前提のみ）
- TODO: 具体の認可方式（トークン/ロール/ネットワーク境界）

---

## 11. Health Report (F-62)

### 11.1 Metrics (Examples)
- drift推移（週平均/最大）
- step/leap回数
- ntp_unsynced時間
- venue suspect回数・uncertainty推移
- budget違反回数
- schema drift検出回数
- quarantine件数
- tls failure相関（時計起因っぽい割合）

### 11.2 Output
- 自動生成（ジョブ/cronは実装側）
- EvidenceBundle と相互リンク可能
- TODO: 出力フォーマット（JSON/MD/PDF等）と保存先、リンク方式

---

## 12. Verification

### 12.1 Additional Tests (v1.4)
- Warmup期間中の品質抑制
- EvidenceBundleの内容整合
- EnvProfileでの閾値上書きとアラート抑制
- 注釈の紐付け表示
- レポート生成の集計検証

### 12.2 DoD (v1.4)
- Evidence export ができる
- Warmup制御があり、起動直後事故が起きにくい
- EnvProfileで環境差を吸収できる
- 注釈が残せる
- 週次/月次の健康診断が出せる（設計と出力フォーマット確定）

---

## 13. Capability Index（IDs）
- **F-57** Evidence Bundle / export_time_evidence
- **F-58** Warmup control / startup_phase
- **F-59** Operator annotations / attach_annotation
- **F-60** Env time profile / environment_profile_id
- **F-61** External read-only health (/timez summary)
- **F-62** Weekly/Monthly health report
