# 🔐 Secrets読み取りのみ許可
path "secret/data/*" {
  capabilities = ["read"]
}

path "secret/metadata/*" {
  capabilities = ["read"]
}

# 🔍 AppRoleのロールID・Secret IDの取得（取得のみ）
path "auth/approle/role/*/role-id" {
  capabilities = ["read"]
}

path "auth/approle/role/*/secret-id" {
  capabilities = ["update"]
}

# 🧾 Vaultのヘルス・ステータス確認
path "sys/health" {
  capabilities = ["read"]
}

# 🔍 認証情報確認用（自身のToken情報確認）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
