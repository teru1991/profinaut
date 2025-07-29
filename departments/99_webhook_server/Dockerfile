FROM python:3.11-slim

# 環境変数を明示（例: UTF-8 文字化け防止）
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

WORKDIR /app

# requirements.txt に依存を分離（推奨）
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# アプリコードをコピー
COPY main.py .

# ポートを明示（FastAPIのデフォルトは8000ですが、ここでは5000）
EXPOSE 5000

# 起動コマンド
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "5000"]
