# ws-ingest run

## 起動コマンド（env例つき）

リポジトリルートで実行:

```bash
# 例：GMOだけ動かす（supervisor側で gmocoin に絞っている構成）
export UCEL_COVERAGE_DIR="ucel/coverage"
export UCEL_RULES_DIR="ucel/crates/ucel-ws-rules/rules"
export UCEL_STORE_PATH="/tmp/ucel-ws-subscriber.sqlite"
export UCEL_JOURNAL_DIR="/tmp/ucel-wal"
export UCEL_FSYNC_MODE="relaxed"        # devはrelaxed、運用はbalanced推奨
export UCEL_RECV_QUEUE_CAP="4096"
export UCEL_MAX_FRAME_BYTES="4194304"
export UCEL_MAX_INFLIGHT_PER_CONN="64"
export UCEL_CONNECT_TIMEOUT_SECS="10"
export UCEL_IDLE_TIMEOUT_SECS="30"
export UCEL_RECONNECT_STORM_WINDOW_SECS="30"
export UCEL_RECONNECT_STORM_MAX="12"
export UCEL_MAX_CONNECTIONS_PER_EXCHANGE="512"
export UCEL_ENABLE_PRIVATE_WS="false"
export RUST_LOG="info"

cargo run -p ucel-ws-subscriber
```

## チェックリスト（具体）

### 4.1 起動直後（5分以内に確認）

- ログに `ucel-ws-subscriber starting` が出る
- ログに symbols + ops（symbols数、ops数）が出る
- `UCEL_STORE_PATH` の sqlite が作成される
- `subscriptions` テーブルがある
- `state='pending'` → 実行後しばらくで inflight/active が増える
- `UCEL_JOURNAL_DIR` に WAL ファイルが増える（サイズが増加し続ける）
- 例：`ls -lh /tmp/ucel-wal` で更新時刻と容量が動く

### 4.2 稼働中（15分程度）

- active が 0 のままにならない（少なくとも ticker は active になる）
- deadletter が大量に増えていない
- op_id mismatch、symbol変換ミス、subscribe失敗があると増える
- CPUが張り付かない（受信ループで重い処理をしていない＝OK）

### 4.3 再接続（手動で確認）

- ネットワークを一時遮断（またはWS先が落ちた想定）で `connect error -> reconnecting` のログが出る
- 復旧後、再接続して WAL の増加が再開する
- store の active/inflight が pending に戻ってから再び active へ進む（続きから）

### 4.4 安全停止（意図的テスト）

- `UCEL_MAX_FRAME_BYTES` を極端に小さくして起動 → `frame too large -> stop` 相当で停止する（破損より停止）
- WALディレクトリが書き込み不可の場合（権限/容量）に停止する（append-firstの失敗停止）

## テスト実行コマンド

```bash
cargo test -p ucel-transport ws_connection_e2e
cargo test -p ucel-cex-gmocoin ws_adapter_contract
```
