#!/usr/bin/env bash
set -euo pipefail

# 使い方:
#   merge_env.sh <SOURCE_ENV> <TARGET_ENV> <APPEND_ENV>
# 例:
#   merge_env.sh .env docker/vault/.env docker/vault/scripts/env.generated

SRC="${1:?source .env path required}"
DST="${2:?target .env path required}"
APP="${3:?append .env path required}"

# 1) マスターをコピー（初回生成 or 上書きしたくないなら存在時はバックアップ）
mkdir -p "$(dirname "$DST")"
if [[ -f "$DST" ]]; then
  cp -f "$DST" "${DST}.bak.$(date +%Y%m%d%H%M%S)"
fi
cp -f "$SRC" "$DST"

# 2) 追記するキー一覧を抽出（#で始まる行、空行は無視）
mapfile -t KEYS < <(grep -E '^[A-Za-z_][A-Za-z0-9_]*=' "$APP" | cut -d= -f1 | sort -u)

# 3) 既存の同名キーを削除
for k in "${KEYS[@]}"; do
  # ^KEY= の行を消す（コメントは残す）
  sed -i.bak "/^${k}=.*/d" "$DST"
done
rm -f "${DST}.bak"

# 4) 追記
echo "" >> "$DST"
echo "# ---- appended by merge_env.sh ($(date -Iseconds)) ----" >> "$DST"
cat "$APP" >> "$DST"

echo "✅ merged: $SRC + $APP → $DST"
