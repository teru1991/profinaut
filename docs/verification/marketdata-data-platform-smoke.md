# MarketData Data Platform Smoke Verification (DP-001..DP-010)

## 1. 前提（必要コマンド/環境）

### 必要コマンド
- `bash`
- `curl`
- `python` (3.11+)
- `pytest`（任意。API起動前の回帰チェックに利用）
- `rg`（任意。静的チェックの見やすさ向上）

### 推奨環境変数
- `BASE_URL`（例: `http://127.0.0.1:18080`）
- `DB_DSN`（v0.1は `sqlite:///...` 推奨）
- `BRONZE_FS_ROOT`（例: `/tmp/marketdata-smoke-bronze`）
- `OBJECT_STORE_BACKEND=fs`
- `SILVER_ENABLED=1`

### サンプル入力
- `scripts/verification/sample_raw_trade.json`
- `scripts/verification/sample_raw_unknown.json`

---

## 2. 実行順（0〜10）

### Step-0: 事前確認
- 対象コミットの固定
  - `git rev-parse HEAD`

### Step-1: Check-001（DP-001）Contracts存在確認（静的）
- 目的: Raw/Silver hub schemas の存在確認
- コマンド例:
  - `ls contracts/schemas/common/raw_envelope.schema.json contracts/schemas/marketdata/md_events_json.schema.json contracts/schemas/marketdata/md_trades.schema.json contracts/schemas/marketdata/md_ohlcv.schema.json contracts/schemas/marketdata/md_best_bid_ask.schema.json`

### Step-2: Check-003（DP-004）infra compose 追加ファイル（静的）
- 目的: MinIO+Postgres ローカル導線の存在確認
- コマンド例:
  - `ls infra/compose/marketdata-local.yml docs/runbooks/marketdata-local.md`
  - `rg -n "minio|postgres" infra/compose/marketdata-local.yml`

### Step-3: サービス起動（smoke用）
- 目的: API検証のためローカル起動
- コマンド例:
  - `PYTHONPATH=/workspace/profinaut OBJECT_STORE_BACKEND=fs DB_DSN=sqlite:////tmp/marketdata-smoke.sqlite3 BRONZE_FS_ROOT=/tmp/marketdata-smoke-bronze SILVER_ENABLED=1 python -m uvicorn services.marketdata.app.main:app --host 127.0.0.1 --port 18080`

### Step-4: Check-002（DP-002）/healthz と /capabilities
- 目的: 起動確認 + 縮退確認
- コマンド例:
  - `curl -sS $BASE_URL/healthz`
  - `curl -sS $BASE_URL/capabilities`
- 縮退確認（別起動でDB/Storage未設定）:
  - `curl -sS http://127.0.0.1:18081/capabilities`（`degraded=true` と理由）

### Step-5: Check-004（DP-005）DBメタテーブル存在確認
- 目的: migration適用後テーブルの存在確認
- コマンド例（SQLite版）:
  - `python -c 'import sqlite3; c=sqlite3.connect("/tmp/marketdata-smoke.sqlite3"); print(c.execute("select name from sqlite_master where type=\"table\"").fetchall())'`

### Step-6: Check-005（DP-006）POST /raw/ingest 保存確認
- 目的: ingest成功レスポンスとDBメタ保存
- コマンド例:
  - `curl -sS -X POST "$BASE_URL/raw/ingest" -H "Content-Type: application/json" --data @scripts/verification/sample_raw_trade.json`
  - `python -c 'import sqlite3; c=sqlite3.connect("/tmp/marketdata-smoke.sqlite3"); print(c.execute("select raw_msg_id, object_key, payload_hash from raw_ingest_meta order by rowid desc limit 3").fetchall())'`

### Step-7: Check-006（DP-003）Bronze実体保存確認
- 目的: object_keyに対応したFS実体確認
- コマンド例:
  - `find "$BRONZE_FS_ROOT" -type f | tail`

### Step-8: Check-007（DP-007）dup/lag/counters観測
- 目的: 重複疑いとカウンタ反映
- コマンド例:
  - 同一 payload を2回POST
  - `curl -sS $BASE_URL/capabilities`

### Step-9: Check-008（DP-008）Silver化 or md_events_json fallback
- 目的: 既知payload→Silver、未知payload→events
- コマンド例（SQLite版）:
  - `python -c 'import sqlite3; c=sqlite3.connect("/tmp/marketdata-smoke.sqlite3"); print(c.execute("select raw_msg_id, received_ts from md_trades order by id desc limit 3").fetchall())'`
  - `python -c 'import sqlite3; c=sqlite3.connect("/tmp/marketdata-smoke.sqlite3"); print(c.execute("select raw_msg_id, event_type from md_events_json order by id desc limit 3").fetchall())'`

### Step-10: Check-009 / Check-010 / Check-011
- Check-009（Replay）:
  - `PYTHONPATH=/workspace/profinaut python -m services.marketdata.app.replay --from 2026-02-16T00:00:00Z --to 2026-02-16T01:00:00Z --db-dsn "$DB_DSN" --bronze-root "$BRONZE_FS_ROOT" --dry_run --parser-version v0.2`
- Check-010（latest APIs）:
  - `curl -sS "$BASE_URL/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy"`
  - `curl -sS "$BASE_URL/ohlcv/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy&timeframe=1m"`
- Check-011（payload非ログ出力）:
  - サービスログに `price|qty|side` がpayload本文として出ていないことを確認

---

## 3. 成功条件（期待値）

- Check-001/003: ファイル存在
- Check-002: `/healthz` と `/capabilities` が200を返す。未設定時は `degraded=true` + 理由あり
- Check-004: `raw_ingest_meta`, `ws_sessions`, `ws_subscriptions` が存在
- Check-005: `/raw/ingest` 成功 + `raw_ingest_meta` に `raw_msg_id/object_key/payload_hash` が記録
- Check-006: `object_key` に対応するFSファイルが存在
- Check-007: 同一payload再送で `dup_suspect`/カウンタ増加を確認
- Check-008: `md_trades` または `md_events_json` に `raw_msg_id` 追跡可能
- Check-009: replayコマンドがsummary（`read_count/silver_count/events_count`）を返す
- Check-010: `/ticker/latest` と `/ohlcv/latest` が `found/stale` など明示応答
- Check-011: payload本文（price/qty/side等）がログに露出しない

---

## 4. 失敗時の切り分け（最小）

1. **サービス接続不可 (`curl`失敗)**
   - `uvicorn` 起動可否、ポート競合、`BASE_URL` ミスを確認
2. **`INGEST_DISABLED` 503**
   - `OBJECT_STORE_BACKEND`, `DB_DSN` を確認
3. **DBクエリ結果が空**
   - `DB_DSN` の実ファイル参照先間違い、ingest payload必須項目不足を確認
4. **Bronzeファイル未生成**
   - `BRONZE_FS_ROOT` と書込み権限確認
5. **replay件数0**
   - `--from/--to` と Bronze partition 時刻の範囲ズレを確認
6. **latest APIが404/503**
   - 404: Silver未投入（期待動作）
   - 503: DB未設定/到達不可
