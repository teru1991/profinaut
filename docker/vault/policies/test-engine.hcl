# ============================
# 🔁 Policy: test-engine
# 用途: シグナル戦略の検証、Bot再現テスト、イベントトリガー型のシナリオ実行に必要なVaultアクセス権限
# ============================

# Bot構成の読み取り（過去のBot設定を元にテストを再現）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 戦略設定・パラメータの読み取り（テスト対象戦略の再現）
path "secret/data/strategies/*" {
  capabilities = ["read", "list"]
}

# テストエンジン用の構成（期間、イベント条件、ロジック指定など）
path "secret/data/config/test-engine/*" {
  capabilities = ["read", "list"]
}

# パフォーマンス情報の読み取り（フォワード比較、検証評価）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# データ取得設定の参照（どの取得元を再現するか）
path "secret/data/config/data-fetcher/*" {
  capabilities = ["read"]
}

# 自身のVaultトークン状態の確認（死活監視）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
