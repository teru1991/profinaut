# ============================
# 🤖 Policy: auto-ops
# 用途: Botのスコア・異常状態・トレード成績に応じた自動起動/停止/再構成を制御するためのアクセス
# ============================

# Bot構成の参照（自動制御対象のBotを特定）
path "secret/data/bots/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 戦略情報・パラメータ取得（昇格戦略への切り替え判断に使用）
path "secret/data/strategies/*" {
  capabilities = ["read", "list"]
}

# パフォーマンス記録の取得（損益・ドローダウンに基づく判断）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# 自動制御の条件・ルール設定（スコア閾値、異常検出条件など）
path "secret/data/config/auto-ops/*" {
  capabilities = ["read", "list"]
}

# トレードノート・注釈の取得（停止理由や昇格根拠の取得）
path "secret/data/notebooks/trade-notes/*" {
  capabilities = ["read"]
}

# Vaultトークンの監視（死活・TTL切れ対応）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
