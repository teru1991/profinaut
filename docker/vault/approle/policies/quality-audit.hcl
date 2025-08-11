# ============================
# 🧪 Policy: quality-audit
# 用途: 取得データの欠損・異常・重複を検知・遮断し、データ品質を監査・スコアリングするためのアクセス
# ============================

# Bot構成の参照（監査対象のデータが紐づくBot特定に使用）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# データ取得部門の設定（対象データソース・取得仕様など）
path "secret/data/config/data-fetcher/*" {
  capabilities = ["read", "list"]
}

# 品質検査ルール（欠損・外れ値・相関チェック・連続ゼロなど）
path "secret/data/config/data-quality/*" {
  capabilities = ["read", "list"]
}

# 検出結果・スコア・警告フラグの記録（データ品質レポート）
path "secret/data/audit/data-quality/results/*" {
  capabilities = ["read", "create", "update", "list"]
}

# Vaultトークン状態の確認（監視・通知対応）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
