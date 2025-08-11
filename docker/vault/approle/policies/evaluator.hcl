# ============================
# 📊 Policy: evaluator
# 用途: 各Bot・戦略のパフォーマンスを評価し、昇格/除外の判断やスコア記録を行う
# ============================

# Bot構成情報の読み取り（対象Botの評価対象・属性確認）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 戦略パラメータ・バージョンの取得（評価軸や相関分析に使用）
path "secret/data/strategies/*" {
  capabilities = ["read", "list"]
}

# 評価指標・複合スコア式などの定義設定
path "secret/data/config/evaluator/*" {
  capabilities = ["read", "list"]
}

# 評価結果の記録（戦略スコア、Botスコア、相関行列、信頼性）
path "secret/data/evaluation/results/*" {
  capabilities = ["read", "create", "update", "list"]
}

# パフォーマンス記録の取得（損益・DD・勝率など）
path "secret/data/performance/*" {
  capabilities = ["read"]
}

# トークン状態の確認（監視・死活チェック）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
