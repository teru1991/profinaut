# ============================
# 📡 Policy: signal-engine
# 用途: 戦略・相場データに基づいたシグナル生成に必要な設定・認証情報へのアクセス
# ============================

# Bot構成の読み取り（signal_engineはBot IDごとに戦略処理を行う）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# シグナル生成に必要な戦略設定・パラメータ情報の取得（各Bot固有）
path "secret/data/strategies/*" {
  capabilities = ["read", "list"]
}

# シグナル計算ロジック用の設定やテンプレートの読み取り
path "secret/data/config/signal-engine/*" {
  capabilities = ["read", "list"]
}

# データ取得設定（データ取得部門の設定を参照する場合）
path "secret/data/config/data-fetcher/*" {
  capabilities = ["read"]
}

# 自分自身のToken状態確認（死活・監視用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
