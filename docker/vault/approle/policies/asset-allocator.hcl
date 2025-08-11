# ============================
# 💰 Policy: asset-allocator
# 用途: 各Botへの資産割当・制御、およびクラスタ/想定損失ベース配分のための設定アクセス
# ============================

# Bot構成の参照（対象Botと構成情報の取得）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# Botごとの割当資産・想定リスク情報（読み取り＋更新）
path "secret/data/allocations/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 資産配分エンジンの設定（静的・動的配分、クラスタ構成、最大ドローダウンなど）
path "secret/data/config/asset-allocator/*" {
  capabilities = ["read", "list"]
}

# 各Botの損益評価・ドローダウン情報（想定損失を考慮した制御に使用）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# Vaultトークン情報の取得（死活チェック・監視用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
