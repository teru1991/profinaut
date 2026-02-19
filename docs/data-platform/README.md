# Profinaut Data Platform SSOT (Free OSS Only)

本ディレクトリは Profinaut データ基盤の SSOT（Single Source of Truth）です。対象は **Raw-first Bronze SSOT → Silver → Gold → Serving** の全体設計であり、技術選定は「完全無料で使える OSS（Apache-2.0/BSD/MIT/PostgreSQL License/LGPL）」に固定します。

## Scope
- Ingestion から Serving までのデータフロー
- Bronze（生データ）中心のトレーサビリティ契約
- DQ（Data Quality）/ Security / Ops runbook
- 技術選定 ADR（採用・非採用理由）

## Non-negotiable policy
- **Object Storage**: SeaweedFS（第一推奨, Apache-2.0）
  - 代替: RustFS（Apache-2.0）
- **Lakehouse table format**: Apache Iceberg（Apache-2.0）
- **OLAP**: ClickHouse（Apache-2.0）
- **OLTP**: PostgreSQL（PostgreSQL License）
- **Cache**: Valkey（BSD-3-Clause）
- **Recompute / local forensic**: DuckDB（MIT）
- **Not adopted**: MinIO / Redis

## Document map
- `architecture.md`: end-to-end 構成と責務分離
- `storage-contracts.md`: Bronze レコード契約、secret scrub、idempotency、RawRef
- `lakehouse-layout.md`: Iceberg namespace/table/partition 設計
- `serving-stores.md`: ClickHouse/PostgreSQL/Valkey の使い分け
- `data-quality.md`: 検証レイヤと SLO
- `security-and-secrets.md`: 秘匿情報非永続化方針
- `ops-runbook.md`: 運用手順（障害対応、再計算、復旧）
- `adrs/*.md`: 技術選定の意思決定記録

## Design principles
1. Bronze is immutable SSOT（監査・再計算の起点）
2. Silver/Gold は deterministic transform のみ
3. Serving 層は再構築可能な派生ストアとみなす
4. 秘匿情報は永続化しない（denylist + schema gate）
5. 1イベントは canonical ID と idempotency key で重複制御

## ADR index
- ADR-0001: Lakehouse format = Iceberg
- ADR-0002: Object storage = SeaweedFS（RustFS fallback）, MinIO 非採用
- ADR-0003: Serving stores = ClickHouse + PostgreSQL + Valkey（Redis 非採用）
- ADR-0004: Canonical IDs & dedupe
- ADR-0005: Secrets non-persistence
