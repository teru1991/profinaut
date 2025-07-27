# ============================
# ⚖️ Policy: risk-reward
# 用途: 各Botのパフォーマンス・損益に応じたリスク調整および報酬最適化のための設定アクセス
# ============================

# Bot構成情報の参照（対象Bot、タグ、構成を取得）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# Botごとの損益履歴、評価指標、パフォーマンスログ（読み取り専用）
path "secret/data/performance/*" {
  capabilities = ["read", "list"]
}

# 最適化ロジックや閾値の設定（risk-reward用のパラメータ）
path "secret/data/config/risk-reward/*" {
  capabilities = ["read", "list"]
}

# シグナル生成戦略の設定も一部参照（損益に基づくリスク制御のため）
path "secret/data/strategies/*" {
  capabilities = ["read"]
}

# 自身のVaultトークン状態（監視・死活チェック）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
