# Data Quality

## 1. Quality gates
- Bronze gate:
  - JSON schema validation
  - required metadata check
  - secret scrub gate
- Silver gate:
  - 型/単位/時刻正規化
  - duplicate rate threshold
  - null ratio threshold
- Gold gate:
  - business invariants（例: OHLC 整合、negative volume 禁止）

## 2. DQ metrics
- ingestion_lag_seconds (p50/p95)
- duplicate_ratio
- schema_reject_count
- scrub_reject_count
- transform_error_count
- serving_freshness_seconds

## 3. SLO example
- Bronze ingest success >= 99.9%
- Silver transform success >= 99.5%
- Gold publish freshness <= 120s (p95)

## 4. Incident handling
1. 異常メトリクス検知
2. 影響 table/partition を特定
3. Bronze を起点に Silver/Gold 再計算
4. DQ 再検証後に serving refresh
