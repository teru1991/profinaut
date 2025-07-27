import os
import hvac
import requests

vault = hvac.Client(url=os.getenv("VAULT_ADDR"))
auth = vault.auth_approle.login(
    role_id=os.getenv("WEBHOOK_SERVER_ROLE_ID"),
    secret_id=os.getenv("WEBHOOK_SERVER_SECRET_ID"),
)
vault.token = auth["auth"]["client_token"]

channel = os.getenv("WEBHOOK_CHANNEL", "alerts")
secret = vault.secrets.kv.v2.read_secret_version(path=f"discord_hooks/{channel}")
url = secret["data"]["data"]["DISCORD_WEBHOOK_URL"]

resp = requests.post(
    url,
    json={
        "username": "Profinaut Notifier",
        "content": "✅ Webhook送信テスト成功（Vault連携）",
    },
)
print(f"Sent test: {resp.status_code}")
