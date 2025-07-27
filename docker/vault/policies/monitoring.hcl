# ============================
# 📡 Policy: monitoring
# 用途: Lokiログ、Prometheusメトリクス、Alertmanager通知のための設定・監視状態取得
# ============================

# Bot一覧・状態の読み取り（監視対象Botのメタ情報を取得）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# メトリクスや通知対象サービスの構成
path "secret/data/config/monitoring/*" {
  capabilities = ["read", "list"]
}

# Alertmanager通知設定（Discord Webhook設定・ルーティングなど）
path "secret/data/config/alertmanager/*" {
  capabilities = ["read"]
}

# Lokiログ関連設定（ルール、ラベル、ログフォーマット）
path "secret/data/config/loki/*" {
  capabilities = ["read"]
}

# Vault Tokenの監視（AppRoleトークンのTTLチェックなど）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

# Vaultのauditログ設定を確認（任意／高度監視用）
path "sys/audit" {
  capabilities = ["read", "list"]
}
