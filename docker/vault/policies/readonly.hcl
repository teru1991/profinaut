# ============================
# 👁️ Policy: readonly
# 用途: ログ・メトリクス監視専用の読み取り専用Vault権限（変更不可）
# ============================

# Bot構成の参照（メトリクス可視化のラベル付与など）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 各部門の設定（読み取り専用）
path "secret/data/config/*" {
  capabilities = ["read", "list"]
}

# APIキーなどの読み取り（ただし keys の中でも限定ディレクトリのみ）
path "secret/data/keys/public-readonly/*" {
  capabilities = ["read"]
}

# トークン状態監視（Grafana/PrometheusによるTTLチェック）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
