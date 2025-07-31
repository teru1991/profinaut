FROM python:3.11-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
  curl ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /vault/scripts

COPY vault/scripts/requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY vault/scripts/ ./

ENTRYPOINT ["bash", "init.sh"]
