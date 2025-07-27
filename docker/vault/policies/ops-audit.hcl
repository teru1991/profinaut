# ============================
# 📜 Policy: ops-audit
# 用途: 設定変更・構成差分・操作履歴の記録と検証（署名・ハッシュ・差分比較含む）に必要なアクセス
# ============================

# Bot構成・設定の読み取り（変更前後の比較対象）
path "secret/data/bots/*" {
  capabilities = ["read", "list"]
}

# 操作記録の保存（設定変更・UI操作・CLIコマンドの記録先）
path "secret/data/audit/ops-log/*" {
  capabilities = ["read", "create", "update", "list"]
}

# 差分・署名のルール設定（監査対象のパス、ハッシュ方法、比較対象など）
path "secret/data/config/ops-audit/*" {
  capabilities = ["read", "list"]
}

# 評価結果や昇格判断のログ記録も参照（文脈理解の補助）
path "secret/data/evaluation/results/*" {
  capabilities = ["read"]
}

# トレードノートとのリンク（注釈・根拠追跡）
path "secret/data/notebooks/trade-notes/*" {
  capabilities = ["read"]
}

# Vaultトークン監視（失効・監査自体の動作確認）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
