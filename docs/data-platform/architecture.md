# Data Platform Architecture

## 1. Pipeline overview

```text
Sources (REST/WS/FIX/etc)
  -> Ingestion adapters
  -> Bronze (raw immutable events on SeaweedFS/RustFS)
  -> Silver (normalized + validated on Iceberg)
  -> Gold (domain marts/features on Iceberg)
  -> Serving
      - ClickHouse (OLAP)
      - PostgreSQL (OLTP/API state)
      - Valkey (cache/session/hot keys)
```

## 2. Stage responsibilities
- Bronze:
  - Raw payload + metadata + provenance を変更不可で保持
  - 再処理の単一入力
- Silver:
  - スキーマ正規化、時刻統一、型の厳密化、PII/secret scrub 完了
- Gold:
  - 戦略/API 向けのドメイン指向モデル（OHLCV、book snapshots、execution facts）
- Serving:
  - ユースケース別の read-optimized store

## 3. Storage and compute choices (free OSS only)
- Object storage: SeaweedFS（推奨）/ RustFS（代替）
- Table format: Apache Iceberg
- Batch/incremental recompute: DuckDB + SQL/ETL workers
- OLAP serving: ClickHouse
- OLTP serving: PostgreSQL
- Cache: Valkey

## 4. Why raw-first
- upstream API 変更・バグ時の再現性確保
- transform バージョン差分の比較が容易
- feature/aggregate の再生成コストを局所化

## 5. Explicit non-adoption
- MinIO は本 SSOT では採用しない（ADR-0002）
- Redis は本 SSOT では採用しない（ADR-0003）
