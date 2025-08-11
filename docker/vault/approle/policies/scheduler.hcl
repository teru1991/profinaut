# ============================
# ⏱️ Policy: scheduler
# 用途: タスクのDAG構築、依存関係・スケジューリング管理、ジョブトリガーのVault設定アクセス
# ============================

# Bot構成の参照（ジョブ定義の対象Botや実行条件を把握）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# スケジューラ設定（実行間隔、依存ルール、トリガー条件など）
path "secret/data/config/scheduler/*" {
  capabilities = ["read", "list"]
}

# タスクテンプレート・DAG構造の読み取り（ワーカー・ジョブ制御と連携）
path "secret/data/config/worker-controller/*" {
  capabilities = ["read", "list"]
}

# GitHub Actions連携やCI/CDトリガー設定（GitHub Tokenなどは別途制御）
path "secret/data/config/ci-pipeline/*" {
  capabilities = ["read", "list"]
}

# Vaultトークン状態の確認（メトリクス・失効検知）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
