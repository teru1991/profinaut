import os
import hvac
import httpx
from fastapi import FastAPI, Request

app = FastAPI()

# --- Vault認証（AppRole使用推奨） ---
vault = hvac.Client(url=os.getenv("VAULT_ADDR"))

if "VAULT_TOKEN" in os.environ:
    vault.token = os.getenv("VAULT_TOKEN")
else:
    role_id = os.getenv("WEBHOOK_SERVER_ROLE_ID")
    secret_id = os.getenv("WEBHOOK_SERVER_SECRET_ID")
    vault.token = vault.auth_approle.login(role_id, secret_id)["auth"]["client_token"]


@app.post("/webhook/{channel}")
async def forward_webhook(channel: str, request: Request):
    try:
        secret = vault.secrets.kv.v2.read_secret_version(
            path=f"discord_hooks/{channel}"
        )
        webhook_url = secret["data"]["data"]["DISCORD_WEBHOOK_URL"]
    except Exception as e:
        return {"error": str(e)}, 404

    data = await request.json()
    async with httpx.AsyncClient() as client:
        resp = await client.post(webhook_url, json=data)

    return {"status": resp.status_code}
