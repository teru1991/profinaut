# ============================
# 📝 Policy: bot-log
# 用途: Botの動作ログ記録、損益・戦略メモ・注釈付きトレードノートの自動生成に必要な設定・記録領域アクセス
# ============================

# Bot構成の参照（ログ記録対象の特定、Bot種別や戦略タグ取得）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# Botの戦略・シグナル・構成に関する読み取り（ノート注釈生成に使用）
path "secret/data/strategies/*" {
  capabilities = ["read"]
}

# トレードノートの自動記録（各トレードの文脈・要因・損益など）
path "secret/data/notebooks/trade-notes/*" {
  capabilities = ["read", "create", "update", "list"]
}

# ノート生成ルールやテンプレート（注釈文、フィールド構成など）
path "secret/data/config/bot-log/*" {
  capabilities = ["read", "list"]
}

# Vaultトークン状態の確認（監視・Prometheus用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
