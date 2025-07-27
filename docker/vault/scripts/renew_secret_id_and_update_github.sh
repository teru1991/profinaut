#!/bin/bash
set -euo pipefail

GITHUB_REPO="studiokeke/profinaut"
SECRET_NAME="PROFINAUT_ADMIN_SECRET_ID"
ROLE_NAME="profinuat_admin"
VAULT_ADDR=${VAULT_ADDR:-"https://vault.profinaut.studiokeke.com:8200"}

ROLE_ID=${PROFINAUT_ADMIN_ROLE_ID}
SECRET_ID=${PROFINAUT_ADMIN_SECRET_ID}

if [[ -z "${GH_TOKEN:-}" ]]; then
  echo "❌ GH_TOKEN（GitHub PAT）が未設定です"
  exit 1
fi

# Vault login
VAULT_TOKEN=$(curl -s --request POST "$VAULT_ADDR/v1/auth/approle/login" \
  --data "{\"role_id\":\"$ROLE_ID\",\"secret_id\":\"$SECRET_ID\"}" \
  | jq -r .auth.client_token)

# SECRET_ID 再発行
NEW_SECRET_ID=$(curl -s --header "X-Vault-Token: $VAULT_TOKEN" \
  --request POST "$VAULT_ADDR/v1/auth/approle/role/$ROLE_NAME/secret-id" \
  | jq -r .data.secret_id)

# GitHub Secret 更新
echo "$NEW_SECRET_ID" | gh secret set "$SECRET_NAME" \
  --repo "$GITHUB_REPO" \
  --app actions \
  --body - <<< "$NEW_SECRET_ID"

echo "✅ GitHub Secrets ($SECRET_NAME) を更新しました"