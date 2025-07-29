FROM python:3.11-slim

# ✅ OSパッケージ: curl を含む最小構成の追加
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /vault/scripts

# ✅ Python依存関係インストール
COPY vault/scripts/requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# ✅ スクリプトのコピー
COPY vault/scripts/ ./

ENTRYPOINT ["bash", "init.sh"]
