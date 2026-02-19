# UCEL-REGISTRY-POSTB-003 整合チェック

- SSOT: `docs/exchanges/gmocoin/catalog.json`
- 実装: `services/marketdata/app/registry.py`
- 起動時に `load_venue_registry("gmocoin")` を実行し、ID重複・必須項目・型定義破損を検証。
- 検証失敗時は起動エラー化し、`/healthz` の `registry_ok` と `degraded_reasons` に反映。
- `requires_auth` は `visibility=private` のみを根拠に判定（推測なし）。
- `op` は `id` と `operation` 文字列規約から固定マッピング。
