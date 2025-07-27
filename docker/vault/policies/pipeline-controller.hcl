# ============================
# 🧩 Policy: pipeline-controller
# 用途: Kafkaストリーム処理の制御（Consumer Group監視、再送制御、キュー監視）に必要な設定アクセス
# ============================

# Bot構成の参照（どのBotがどのトピックを使用しているかを把握）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# パイプライン制御の設定（各トピックの監視対象、再送ルールなど）
path "secret/data/config/pipeline-controller/*" {
  capabilities = ["read", "list"]
}

# Kafka接続設定（ブローカー、トピック名、コンシューマ設定など）
path "secret/data/config/kafka/*" {
  capabilities = ["read", "list"]
}

# パフォーマンス記録（Lagや遅延発生状況の取得に利用可能）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# Vaultトークン状態の確認（死活監視）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
