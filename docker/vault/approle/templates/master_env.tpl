# ---- rendered by Vault Agent (DO NOT COMMIT) ----
# 共通メタ
ENV=production
TIMEZONE=Asia/Tokyo
LOG_LEVEL=info

# 例: KV v2 (secret/data/profinaut/common) から読み出し
{{ with secret "secret/data/profinaut/common" -}}
CF_ACCESS_CLIENT_ID={{ .Data.data.METRICS_CF_ACCESS_CLIENT_ID }}
CF_ACCESS_CLIENT_SECRET={{ .Data.data.METRICS_CF_ACCESS_CLIENT_SECRET }}
DISCORD_ALERT_WEBHOOK={{ .Data.data.DISCORD_ALERT_WEBHOOK }}
{{- end }}

# 例: 各サービスの AppRole を KV に格納しておき、ここで埋める方式
{{ with secret "secret/data/profinaut/approles/bot_manager" -}}
BOT_MANAGER_ROLE_ID={{ .Data.data.ROLE_ID }}
BOT_MANAGER_SECRET_ID={{ .Data.data.SECRET_ID }}
{{- end }}
{{ with secret "secret/data/profinaut/approles/signal_engine" -}}
SIGNAL_ENGINE_ROLE_ID={{ .Data.data.ROLE_ID }}
SIGNAL_ENGINE_SECRET_ID={{ .Data.data.SECRET_ID }}
{{- end }}
{{ with secret "secret/data/profinaut/approles/trade_executor" -}}
TRADE_EXECUTOR_ROLE_ID={{ .Data.data.ROLE_ID }}
TRADE_EXECUTOR_SECRET_ID={{ .Data.data.SECRET_ID }}
{{- end }}
{{ with secret "secret/data/profinaut/approles/webhook_server" -}}
WEBHOOK_SERVER_ROLE_ID={{ .Data.data.ROLE_ID }}
WEBHOOK_SERVER_SECRET_ID={{ .Data.data.SECRET_ID }}
{{- end }}
