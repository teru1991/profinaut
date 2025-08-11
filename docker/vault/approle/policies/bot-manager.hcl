# ============================
# 🛡️ Policy: bot-manager
# 用途: Bot一覧・構成・状態制御のためのVaultアクセス権限
# ============================

# Bot一覧や設定の取得（bot_manager による全体管理）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# Botの構成設定（例: タグ, 種別, 状態）への読み取り・更新
path "secret/data/config/bot-manager/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 共通システム設定（例: scheduler設定、制御フラグなど）への読み取り
path "secret/data/system/*" {
  capabilities = ["read", "list"]
}

# 自分自身のTokenに関する情報取得（監視用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

# Vaultの監査対象確認（必要に応じて）
# path "sys/audit" {
#   capabilities = ["read", "list"]
# }
