import os
import json
import hvac
import requests

# Vault接続
vault = hvac.Client(url=os.environ["VAULT_ADDR"])

# AppRoleログイン（hvac >=1.0以降対応）
role_id = os.environ["VAULT_ROLE_ID"]
secret_id = os.environ["VAULT_SECRET_ID"]

auth_response = vault.auth.approle.login(role_id=role_id, secret_id=secret_id)

vault.token = auth_response["auth"]["client_token"]

# Webhook URL取得（KV v2）
channel = os.environ.get("WEBHOOK_CHANNEL", "alerts")
webhook_data = vault.secrets.kv.v2.read_secret_version(path=f"webhook/{channel}")
webhook_url = webhook_data["data"]["data"]["url"]

# ペイロード送信
payload = json.loads(os.environ["TEST_PAYLOAD"])
response = requests.post(webhook_url, json=payload)

# 結果表示
print(f"✅ Webhook送信結果: {response.status_code}")
print(response.text)
