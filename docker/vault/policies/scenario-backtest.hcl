# ============================
# 🎯 Policy: scenario-backtest
# 用途: 戦略の複雑な条件付き検証・トリガーシナリオ定義・比較評価に必要な構成・記録アクセス
# ============================

# Bot構成情報（テスト対象戦略や構成条件の確認）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 戦略パラメータ・設定（バージョン間比較、トリガー検証）
path "secret/data/strategies/*" {
  capabilities = ["read", "list"]
}

# シナリオ定義（イベント発火条件、複数戦略の並列テスト）
path "secret/data/config/scenario-backtest/*" {
  capabilities = ["read", "list"]
}

# テスト結果・ヒートマップ記録（スコア比較、トリガー通過率など）
path "secret/data/backtest/scenario-results/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 評価結果（従来のスコアリングと比較し、改善度を把握）
path "secret/data/evaluation/results/*" {
  capabilities = ["read"]
}

# Vaultトークン監視（死活・自動再起動判定）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
