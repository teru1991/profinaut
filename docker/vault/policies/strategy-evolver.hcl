# ============================
# 🧬 Policy: strategy-evolver
# 用途: 戦略の生成・進化・評価ループに必要な設定・結果記録へのアクセス権限
# ============================

# Bot構成の読み取り（戦略適用先の条件確認に使用）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 進化対象の戦略構成・パラメータ（読み取り＋更新）
path "secret/data/strategies/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 進化エンジンの構成（進化条件、スコア式、評価基準など）
path "secret/data/config/strategy-evolver/*" {
  capabilities = ["read", "list"]
}

# 評価結果やスコアの記録（進化プロセスの追跡・履歴化）
path "secret/data/evolution/records/*" {
  capabilities = ["read", "create", "update", "list"]
}

# パフォーマンス情報の読み取り（進化評価の指標に使用）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# 自身のVaultトークン情報（監視・死活チェック）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
