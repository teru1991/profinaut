import os
import requests

# 📍 Vault および GitHub OIDC 設定
VAULT_ADDR = os.environ["VAULT_ADDR"]
CERT = os.environ.get("VAULT_CERT", False)
OIDC_PATH = os.environ.get("VAULT_MOUNT", "oidc")
REPO = os.environ["GITHUB_REPO"]
GITHUB_ISSUER = "https://token.actions.githubusercontent.com"

# 🔐 GitHub Actions OIDC Token 取得
req_url = os.environ.get("ACTIONS_ID_TOKEN_REQUEST_URL")
req_token = os.environ.get("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
aud = "vault"  # Vault側と一致させるAudience名

headers = {"Authorization": f"Bearer {req_token}"}
id_token = requests.get(f"{req_url}&audience={aud}", headers=headers).json()["value"]

# ✅ OIDC Auth Method 有効化
requests.post(
    f"{VAULT_ADDR}/v1/sys/auth/{OIDC_PATH}",
    headers={"X-Vault-Token": os.getenv("VAULT_TOKEN", "")},
    json={
        "type": "oidc",
        "config": {"default_lease_ttl": "1h", "max_lease_ttl": "24h"},
    },
    verify=CERT,
)

# 🧑‍💻 GitHub Actions ロール作成
role_data = {
    "role_type": "jwt",
    "user_claim": "actor",
    "bound_audiences": aud,
    "bound_claims": {"repository": REPO},
    "policies": "admin",  # 必要に応じて変更
    "ttl": "1h",
}

r = requests.post(
    f"{VAULT_ADDR}/v1/auth/{OIDC_PATH}/role/github-actions",
    headers={"X-Vault-Token": os.getenv("VAULT_TOKEN", "")},
    json=role_data,
    verify=CERT,
)
r.raise_for_status()
print("✅ GitHub OIDC Role 登録完了")
