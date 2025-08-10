#!/usr/bin/env sh
set -eu

: "${GITHUB_REPO:?GITHUB_REPO is required (e.g. StudioKeKe/profinaut)}"
: "${GH_DISPATCH_TOKEN:?GH_DISPATCH_TOKEN is required (repo-scope PAT)}"
: "${VAULT_ADDR:?VAULT_ADDR is required}"
: "${CF_ACCESS_CLIENT_ID:?CF_ACCESS_CLIENT_ID is required}"
: "${CF_ACCESS_CLIENT_SECRET:?CF_ACCESS_CLIENT_SECRET is required}"

echo "🔎 Checking Vault health at ${VAULT_ADDR} ..."
# Vault 応答待ち
i=0
until curl -sS --fail --cacert /root/origin_ca.pem \
  -H "CF-Access-Client-Id: ${CF_ACCESS_CLIENT_ID}" \
  -H "CF-Access-Client-Secret: ${CF_ACCESS_CLIENT_SECRET}" \
  "${VAULT_ADDR}/v1/sys/health" >/tmp/health.json 2>/dev/null; do
  i=$((i+1))
  [ $i -gt 60 ] && echo "❌ Timeout waiting Vault" && exit 2
  sleep 3
done

if jq -e '.initialized == false' /tmp/health.json >/dev/null 2>&1; then
  echo "🚀 Vault not initialized → send repository_dispatch"
  # GitHub の仕様上、成功時は 204 No Content
  code=$(curl -sS -o /dev/null -w "%{http_code}" -X POST \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: token ${GH_DISPATCH_TOKEN}" \
    "https://api.github.com/repos/${GITHUB_REPO}/dispatches" \
    -d '{"event_type":"vault-first-boot"}')
  if [ "$code" = "204" ]; then
    echo "✅ repository_dispatch sent (204)"
  else
    echo "❌ repository_dispatch failed (HTTP $code)"
    exit 3
  fi
else
  echo "ℹ️  Vault already initialized. No dispatch."
fi
