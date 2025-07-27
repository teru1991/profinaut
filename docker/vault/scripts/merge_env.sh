#!/bin/bash
set -e

# 📍ファイルパス
ENV_FILE="../.env"
GEN_FILE=".env.generated"
TMP_FILE=".env.merged"

# ✅ チェック
if [ ! -f "$GEN_FILE" ]; then
  echo "❌ $GEN_FILE が見つかりません。先に create_approles.py を実行してください。"
  exit 1
fi

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ $ENV_FILE が見つかりません。"
  exit 1
fi

# 🛠 上書きまたは追加
echo "🔄 $GEN_FILE の内容を $ENV_FILE にマージします..."

# 古い ROLE_ID / SECRET_ID を除去
cp "$ENV_FILE" "$TMP_FILE"
while read -r line; do
  key=$(echo "$line" | cut -d '=' -f1)
  if grep -q "^$key=" "$TMP_FILE"; then
    sed -i.bak "/^$key=/d" "$TMP_FILE"
  fi
done < "$GEN_FILE"

# マージ
cat "$GEN_FILE" >> "$TMP_FILE"
mv "$TMP_FILE" "$ENV_FILE"

echo "✅ マージ完了: $ENV_FILE が更新されました。バックアップは $ENV_FILE.bak にあります。"
