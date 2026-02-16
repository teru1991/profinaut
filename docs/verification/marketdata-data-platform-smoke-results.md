# MarketData Data Platform Smoke Results (DP-001..DP-010)

## 実施情報

- 実施日 (UTC): 2026-02-16 12:25:41 UTC
- 実施日 (JST): 2026-02-16 21:25:41 JST
- 対象コミットSHA: `a6905900a3ae464f98d57da17b44ea944b87c0de`

## 実行コマンドと主要出力（抜粋）

1. Commit SHA / 時刻
```bash
git rev-parse HEAD
# a6905900a3ae464f98d57da17b44ea944b87c0de

date -u +'%Y-%m-%d %H:%M:%S UTC'
# 2026-02-16 12:25:41 UTC
```

2. Contracts存在
```bash
ls contracts/schemas/common/raw_envelope.schema.json contracts/schemas/marketdata/md_events_json.schema.json contracts/schemas/marketdata/md_trades.schema.json contracts/schemas/marketdata/md_ohlcv.schema.json contracts/schemas/marketdata/md_best_bid_ask.schema.json
# 5ファイルが存在
```

3. health/capabilities（通常起動）
```bash
curl -sS http://127.0.0.1:18080/healthz
# {"status":"ok",...}

curl -sS http://127.0.0.1:18080/capabilities
# {"ingest_raw_enabled":true,"degraded":false,...}
```

4. capabilities（縮退起動）
```bash
curl -sS http://127.0.0.1:18081/capabilities
# {"ingest_raw_enabled":false,"degraded":true,"degraded_reasons":["STORAGE_NOT_CONFIGURED","DB_NOT_CONFIGURED"],...}
```

5. ingest
```bash
curl -sS -X POST 'http://127.0.0.1:18080/raw/ingest' -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_trade.json
# {"stored":true,"dup_suspect":false,"normalized_target":"md_trades",...}

curl -sS -X POST 'http://127.0.0.1:18080/raw/ingest' -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_trade.json
# {"stored":true,"dup_suspect":true,...}

curl -sS -X POST 'http://127.0.0.1:18080/raw/ingest' -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_unknown.json
# {"stored":true,"normalized_target":"md_events_json","normalized_event_type":"venue.ws_message.gmo"}
```

6. DB確認（SQLite）
```bash
python - <<'PY'
import sqlite3
c=sqlite3.connect('/tmp/marketdata-smoke.sqlite3')
print(c.execute("select raw_msg_id, object_key, payload_hash from raw_ingest_meta order by rowid desc limit 3").fetchall())
print(c.execute("select raw_msg_id, received_ts from md_trades order by id desc limit 3").fetchall())
print(c.execute("select raw_msg_id, event_type from md_events_json order by id desc limit 3").fetchall())
PY
# raw_ingest_meta / md_trades / md_events_json いずれも確認
```

7. Bronze実体
```bash
find /tmp/marketdata-smoke-bronze -type f | tail -n 5
# /tmp/marketdata-smoke-bronze/bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00/part-0001.jsonl
```

8. Replay dry-run
```bash
PYTHONPATH=/workspace/profinaut python -m services.marketdata.app.replay --from 2026-02-16T00:00:00Z --to 2026-02-16T01:00:00Z --db-dsn sqlite:////tmp/marketdata-smoke.sqlite3 --bronze-root /tmp/marketdata-smoke-bronze --dry_run --parser-version v0.2
# {"read_count":1,"silver_count":0,"events_count":1,"skipped_count":0,...}
```

9. latest APIs
```bash
curl -sS 'http://127.0.0.1:18080/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy'
# {"found":true,"price":123.45,"stale":true,...}

curl -sS 'http://127.0.0.1:18080/ohlcv/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy&timeframe=1m'
# {"found":false,...}
```

10. payloadログ露出確認（抜粋）
```text
INFO: 127.0.0.1 ... "POST /raw/ingest HTTP/1.1" 200 OK
...
# payload_json本文（price/qty/side値）はログに出力されず
```

---

## Check Matrix（PASS/FAIL）

| Check ID | 項目 | 結果 | 理由 |
|---|---|---|---|
| Check-001 | Contracts存在 | PASS | 対象5schemaファイル存在を確認。 |
| Check-002 | `/healthz`/`/capabilities`（縮退含む） | PASS | 通常起動で200+`degraded=false`、縮退起動で`degraded=true`+理由を確認。 |
| Check-003 | infra compose 追加ファイル（静的） | PASS | `infra/compose/marketdata-local.yml` と runbook存在、`minio/postgres`定義確認。 |
| Check-004 | DBメタテーブル存在 | PASS | SQLite上で `raw_ingest_meta/ws_sessions/ws_subscriptions` 存在確認。 |
| Check-005 | `/raw/ingest` -> Bronze+DB | PASS | ingest成功レスポンス、`raw_ingest_meta` に `raw_msg_id/object_key/payload_hash` 記録確認。 |
| Check-006 | Bronze実体保存 | PASS | object_keyに対応するFSファイル存在を確認。 |
| Check-007 | dup/lag/counters観測 | PASS | 同一payload再送で `dup_suspect=true`、`/capabilities.ingest_stats` 増加確認。 |
| Check-008 | Silver化 or events fallback | PASS | trade payloadが`md_trades`、unknown payloadが`md_events_json`に入り、`raw_msg_id`追跡可。 |
| Check-009 | Replay動作（dry-run） | PASS | replay dry-run がJSON summary（read/silver/events/skipped）を返却。 |
| Check-010 | `/ticker/latest` `/ohlcv/latest` | PASS | tickerは`found/stale`を返却、ohlcv未データ時は`found=false`で明示。 |
| Check-011 | payload本文をログに出さない | PASS | サービスログ上でpayload本文（price/qty/side値）の露出なし。 |

---

## 補足（失敗時の当たり）

今回 FAIL はありません。将来失敗時は以下を優先確認:
- `BASE_URL` と起動ポート
- `OBJECT_STORE_BACKEND` / `DB_DSN` / `BRONZE_FS_ROOT`
- `--from/--to` と Bronze partition 時刻のズレ
