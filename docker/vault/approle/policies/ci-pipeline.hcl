# ============================
# 🚀 Policy: ci-pipeline
# 用途: CI/CD実行時（GitHub Actionsなど）に必要な最低限の読み取りアクセス
# ============================

# Bot定義や構成の読み取り（ビルド/テスト時に使用）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# CI/CDパイプライン専用設定（タグビルドや自動バージョン）
path "secret/data/config/ci-pipeline/*" {
  capabilities = ["read", "list"]
}

# Trade ExecutorやSignal Engine用に必要な最低限の構成
path "secret/data/config/trade-executor/*" {
  capabilities = ["read"]
}
path "secret/data/config/signal-engine/*" {
  capabilities = ["read"]
}

# APIキーやCI限定Secretsの読み取り（制限された内容のみ）
path "secret/data/keys/ci/*" {
  capabilities = ["read"]
}

# 自分自身のVaultトークン状態確認（GitHub ActionsのTTLなど）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
