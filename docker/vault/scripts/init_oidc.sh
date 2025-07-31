#!/bin/bash
set -e

echo "🚀 Vault OIDC 認証方式の初期化を開始..."

# 1. OIDC auth を有効化（すでに有効ならスキップ）
vault auth list -format=json | jq -e '."oidc/"' >/dev/null 2>&1 || \
  vault auth enable oidc

# 2. OIDC Authの設定
vault write auth/oidc/config \
  oidc_discovery_url="https://token.actions.githubusercontent.com" \
  oidc_client_id="${OIDC_CLIENT_ID}" \
  oidc_client_secret="${OIDC_CLIENT_SECRET}" \
  default_role="github-actions"

# 3. Role定義（GitHub Actions からのアクセス用）
vault write auth/oidc/role/github-actions \
  bound_audiences="${OIDC_CLIENT_ID}" \
  user_claim="sub" \
  policies="vault-admin" \
  ttl="1h" \
  bound_subjects="repo:studiokeke/profinaut:*"

echo "✅ OIDC構成完了"
