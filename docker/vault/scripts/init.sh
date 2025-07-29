#!/bin/bash
set -e

echo "🔐 Profinaut Vault 初期化開始..."

# ✅ Docker Compose により /vault/.env にマウントされている
ENV_FILE="/vault/.env"

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ .env ファイルが見つかりません: $ENV_FILE"
  exit 1
fi

source "$ENV_FILE"

if [[ -z "$VAULT_ADDR" ]]; then
  echo "❌ VAULT_ADDR が未定義です。"
  exit 1
fi

# Vault準備待機
until curl -sk "$VAULT_ADDR/v1/sys/health" | grep -q '"sealed":false'; do
  echo "🕓 Vault not ready yet..."
  sleep 3
done

export VAULT_ADDR
export VAULT_TOKEN=$(grep VAULT_TOKEN "$ENV_FILE" | cut -d '=' -f2)

echo "✅ Vault is ready. Injecting..."
python3 create_policies.py
python3 create_approles.py

# 🔄 .env.generated をマージ
if [ -f "/vault/scripts/env.generated" ]; then
  echo "🔄 .env.generated をマージ中..."
  cat /vault/scripts/env.generated >> "$ENV_FILE"
fi

echo "🎉 完了しました！"
