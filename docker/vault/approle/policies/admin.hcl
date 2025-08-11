# ✅ Secretsの完全操作
path "secret/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# ✅ AppRoleの作成・ID取得・Secret ID生成
path "auth/approle/role/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "auth/approle/role/*/role-id" {
  capabilities = ["read"]
}

path "auth/approle/role/*/secret-id" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# ✅ ポリシー管理（create_policies.py に必要）
path "sys/policies/acl/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# ✅ Token操作（OIDC・Tokenチェック用）
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

# ✅ Vault情報確認（ヘルスチェック・状態確認）
path "sys/health" {
  capabilities = ["read"]
}

# ✅ OIDC 設定用（GitHub OIDCを使用する場合）
path "auth/oidc/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
