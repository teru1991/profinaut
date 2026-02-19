# Serving Stores

## Store roles

### ClickHouse (OLAP)
- Use cases: 時系列分析、集計 API、ダッシュボード
- Data source: 主に Gold（必要に応じて Silver）
- SLA: high-throughput read

### PostgreSQL (OLTP)
- Use cases: API の整合性が必要な参照状態、ジョブ管理、設定・メタデータ
- Data source: Gold/Silver 由来の正規化済み state
- SLA: transactional consistency

### Valkey (Cache)
- Use cases: hot key cache, short-lived materialized views, rate-limit/session
- TTL 前提の非正規キャッシュ
- Cold start 時は ClickHouse/PostgreSQL から再構築

## Non-adoption policy
- Redis は採用しない（Valkey を標準化）
- 単一ストア万能設計は行わない（OLAP/OLTP/cache を分離）

## Invalidation strategy
- write-through ではなく event-driven invalidation
- Gold 更新イベントで Valkey keyspace を invalidate
