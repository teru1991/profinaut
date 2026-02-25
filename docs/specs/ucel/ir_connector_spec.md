# UCEL IR / Disclosure Connector Spec v1.0（固定仕様）
Unified Disclosure/IR/Filings Connector（R）

- Document ID: UCEL-IR-CONNECTOR-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): R（Equity/IR Analytics）
- Depends-on: UCEL-SDK-CORE-SPEC
- Contracts SSOT:
  - 監査：`docs/contracts/audit_event.schema.json`
  - リプレイ参照：`docs/contracts/replay_pointers.schema.json`（任意）
- Goal:
  - 開示/IR/決算/訂正等を、取得元差分（web/html/pdf/api等）を吸収し
  - **出所（provenance）と版管理（revision）**を持つ構造化イベントとして提供する
- Non-goals:
  - スコアリング/NLP/検索エンジン実装詳細
  - 閾値・通知ポリシー（Policy）

---

## 0. 不変原則（Non-negotiable）
1. **Provenance-first**：出所と証拠（原文参照）を必ず残す。  
2. **Immutable facts**：訂正/差替えを前提に履歴を消さない（版を持つ）。  
3. **Canonical event**：内部は統一イベントモデル（schema_version付き）。  
4. **Searchable minimum**：最小メタは必ず構造化して検索可能にする。  
5. **No secrets**：秘密は扱わない（公開情報前提、拡張しても漏洩禁止）。

---

## 1. IRイベントモデル（固定：必須概念）
共通ヘッダ（UCEL SDK準拠）：
- event_uid/trace_id/run_id/adapter_id/schema_version
- event_time（公表時刻が取れるなら）/ recv_time / emit_time
- kind：filing/earnings/presentation/revision/dividend/buyback/guidance/offering 等

必須メタ（固定）：
- issuer_id（銘柄コード等の正規化ID）
- market
- title
- source_ref（URL/文書ID等）
- source_type（web/html/pdf/api/edinet/tdnet 等）
- language
- document_version（訂正/差替え識別）
- hash（本文または主要部分のハッシュ：差替え検知）
- content_ref（保存参照：doc store/raw等）

---

## 2. 版管理（revision：固定）
- "最新だけ保持"は禁止
- document_version と supersedes（前版参照）で履歴を追える
- revisionは kind=revision として発火し、関連付けを持つ

---

## 3. 取得・抽出の責務境界（固定）
adapter責務：
- 取得 → 最小メタ抽出 → 版管理 → イベント化

上位（R/P等）責務：
- 高度NLP、スコアリング、ランキング、検索最適化

---

## 4. 品質（欠損/重複/変化：固定）
- 欠損（取れない/遅い/停止）は標準エラー分類
- 重複は hash + source_ref で判定し event_uid を決定的にする
- "取れたが内容が変"は hash 変化で revision を発火する

---

## 5. 監査・説明責任（固定）
- どの情報がどのsourceから来たか追える（provenance）
- 重要イベント（差替え検知、取得失敗継続）は監査イベント化
- 再現：同じ文書→同じevent_uid生成が可能

---

## 6. Observability（固定カテゴリ）
- documents_discovered_total
- documents_fetched_total / fetch_fail_total（category別）
- parse_unknown_total
- revisions_detected_total
- end_to_end_latency（公開→取得→イベント化）

---

## 7. Versioning（SemVer）
- MAJOR：必須メタ/版管理の破壊
- MINOR：kind追加、メタ追加等の後方互換拡張
- PATCH：表現修正
