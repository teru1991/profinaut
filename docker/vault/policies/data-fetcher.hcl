# ============================
# 📥 Policy: data-fetcher
# 用途: 仮想通貨・株式・為替・オンチェーンなど多様なデータ取得に必要なAPIキー・取得設定へのアクセス
# ============================

# Bot一覧の参照（Botに紐づくデータ取得設定の特定に使用）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# データ取得設定（各データソースごとの仕様、周期、リトライルールなど）
path "secret/data/config/data-fetcher/*" {
  capabilities = ["read", "list"]
}

# APIキーや接続先ノード、WebSocketエンドポイントなどの秘密情報
path "secret/data/keys/data-fetcher/*" {
  capabilities = ["read"]
}

# 整合性・異常検出ルール（データ欠損時の遮断、重複検出など）
path "secret/data/config/data-quality/*" {
  capabilities = ["read"]
}

# Vaultトークン状態（監視・TTL確認）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
