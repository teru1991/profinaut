FROM python:3.11-slim

WORKDIR /app

# Pythonスクリプトをコピー（ビルド時に自動で含める用）
COPY test_webhook.py .

# 必要なライブラリをインストール
RUN pip install "hvac==1.1.1" requests


# ENTRYPOINTは docker-compose.yml 側で制御するため指定しません
