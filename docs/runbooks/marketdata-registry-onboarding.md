# Marketdata: 取引所追加手順（catalog SSOT）

1. `docs/exchanges/<venue>/catalog.json` を追加/更新する。  
   - `rest_endpoints` / `ws_channels` / `data_feeds` を記述
   - `id` は全体で一意
   - `visibility` は `public|private`
2. `services/marketdata/app/registry.py` の op マッピング規約で `OpName` に落ちることを確認する。
3. 未対応 op は `supported=false` のまま明示されるため、adapter 実装後にサポート集合を更新する。
4. 必要に応じて `MARKETDATA_CONNECTION_POLICIES` で connection 単位の
   `allowed_ops` / `failover_policy` / `key_scope` を上書きする。
5. `services/marketdata/tests/test_registry.py` と既存 testkit を実行して検証する。
