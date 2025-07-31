#!/bin/bash
set -e

echo "🔐 Profinaut Vault 初期化開始..."

ENV_FILE="/vault/.env"
CERT_FILE="/vault/cert/origin_ca_rsa_root.pem"

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ .env ファイルが見つかりません: $ENV_FILE"
  exit 1
fi

source "$ENV_FILE"

if [[ -z "$VAULT_ADDR" ]]; then
  echo "❌ VAULT_ADDR が未定義です。"
  exit 1
fi

vault_host=$(echo "$VAULT_ADDR" | sed -E 's#https?://([^:/]+).*#\1#')

# ✅ DNS & 接続確認（Cloudflare Tunnel経由）
echo "🌐 Vault接続確認: $VAULT_ADDR"
curl -v --connect-timeout 10 --cacert "$CERT_FILE" "$VAULT_ADDR/v1/sys/health" || {
  echo "❌ Vault への HTTPS 接続失敗。Cloudflare Tunnel や Vault 起動状態を確認してください。"
  exit 2
}

# ✅ sealed が false になるまで待機
echo "⌛ Vault ready 待機中..."
until curl -s --cacert "$CERT_FILE" "$VAULT_ADDR/v1/sys/health" | grep -q '"sealed":false'; do
  echo "🕓 Vault not ready yet..."
  sleep 3
done

echo "✅ Vault is ready! → ポリシーと AppRole を投入します..."

export VAULT_ADDR
export VAULT_TOKEN=$(grep "^VAULT_TOKEN=" "$ENV_FILE" | cut -d '=' -f2)

# ✅ ポリシーと AppRole の投入
python3 create_policies.py
python3 create_approles.py

# ✅ .env.generated をマージ
GENERATED_ENV="/vault/scripts/env.generated"
if [ -f "$GENERATED_ENV" ]; then
  echo "🔄 .env.generated をマージ中..."
  TMP_ENV="/vault/.env.tmp"
  cp "$ENV_FILE" "$TMP_ENV"
  while read -r line; do
    key=$(echo "$line" | cut -d '=' -f1)
    sed -i "/^${key}=.*/d" "$TMP_ENV"
  done < "$GENERATED_ENV"
  cat "$GENERATED_ENV" >> "$TMP_ENV"
  mv "$TMP_ENV" "$ENV_FILE"
  echo "✅ マージ完了: $ENV_FILE"
else
  echo "⚠️ AppRole 環境変数出力 (.env.generated) が見つかりません"
fi

echo "🎉 Vault 初期化完了"
