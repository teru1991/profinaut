# ============================
# 🌐 Policy: webhook-handler
# 用途: Discord通知Webhook、Vault通知Webhook、Alert情報受信時のSecret参照
# ============================

# 通知送信先・Webhookルーティング構成の読み取り
path "secret/data/config/alertmanager/*" {
  capabilities = ["read"]
}
path "secret/data/config/webhook/*" {
  capabilities = ["read"]
}

# Discord通知用トークン・Webhook URLの読み取り
path "secret/data/keys/webhook-discord/*" {
  capabilities = ["read"]
}

# トークン状態確認（トークンTTL監視やVault再取得のため）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
