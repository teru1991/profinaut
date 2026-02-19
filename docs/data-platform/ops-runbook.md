# Ops Runbook

## 1. Daily checks
- ingest lag / reject rate / duplicate ratio を確認
- Iceberg snapshot と retention job の成否確認
- Serving freshness（ClickHouse/PostgreSQL/Valkey）確認

## 2. Replay / backfill
1. 対象範囲を Bronze partition で特定
2. Silver 変換を再実行（version pin）
3. Gold を再生成
4. Serving store を再ロード
5. DQ 指標を再確認

## 3. Object storage failover
- primary: SeaweedFS
- fallback: RustFS
- 切替時は write freeze -> metadata sync -> resume

## 4. Hotfix process
- schema 問題: Bronze gate で reject + incident 発報
- transform 問題: faulty version を停止し last known good に rollback
- cache 問題: Valkey flush + event-driven repopulation

## 5. Disaster recovery
- Bronze を最重要復旧対象とする
- Silver/Gold/Serving は Bronze から再構築
- 復旧後に traceability（RawRef）を監査
