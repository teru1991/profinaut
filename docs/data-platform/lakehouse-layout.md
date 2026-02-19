# Lakehouse Layout (Apache Iceberg)

## 1. Namespace strategy
- `bronze_marketdata`
- `silver_marketdata`
- `gold_marketdata`
- `gold_features`

## 2. Table naming
- Bronze: `bronze_marketdata.{exchange}_{event_type}`
- Silver: `silver_marketdata.{domain}_{entity}`
- Gold: `gold_marketdata.{product}_{view}`

## 3. Partitioning
- 기본: `days(event_time)`
- 高頻度テーブル: `hours(event_time)` + `bucket(32, symbol)`
- private/account 系: `days(event_time)` + `bucket(16, account_id_hash)`

## 4. Snapshot & retention
- Bronze snapshots: 30 days (metadata), data file retention >= 365 days
- Silver snapshots: 14 days
- Gold snapshots: 14 days
- Expire snapshots should never break RawRef lineage

## 5. Schema evolution
- Additive change first
- Rename/drop は互換層を経由
- required column の追加時は default/backfill 計画を ADR に記録

## 6. Recompute with DuckDB
- Bronze files から DuckDB で forensic SQL 実行
- Iceberg テーブル再生成前に row count/hash を比較
