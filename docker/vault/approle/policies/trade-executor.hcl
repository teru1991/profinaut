# ============================
# 🏹 Policy: trade-executor
# 用途: Botによるトレード実行に必要なAPIキー・Bot構成・資産制御情報の取得
# ============================

# Bot構成情報の取得（通貨ペア、取引所、トレードモードなど）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 取引所APIキーなど、Botごとの秘密鍵（取引専用）
path "secret/data/keys/trade-executor/*" {
  capabilities = ["read"]
}

# 資産配分状況の取得（過剰発注を避けるための配分確認）
path "secret/data/allocations/*" {
  capabilities = ["read"]
}

# Bot実行戦略やスリッページ設定などの取得
path "secret/data/config/trade-executor/*" {
  capabilities = ["read", "list"]
}

# Vaultトークン状態（監視・死活チェック用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

