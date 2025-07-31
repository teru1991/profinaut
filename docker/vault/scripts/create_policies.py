import os
import requests

# Vault接続設定
VAULT_ADDR = os.getenv("VAULT_ADDR", "https://vault.profinaut.studiokeke.com:8200")
CERT_PATH = os.getenv("VAULT_CERT", "/vault/cert/origin_ca_rsa_root.pem")

# Cloudflare Access Token
CF_CLIENT_ID = os.getenv("CF_ACCESS_CLIENT_ID")
CF_CLIENT_SECRET = os.getenv("CF_ACCESS_CLIENT_SECRET")

# Vault Token（AppRole / OIDCなどが使われる場合）
VAULT_TOKEN = os.getenv("VAULT_TOKEN")

HEADERS = {
    "CF-Access-Client-Id": CF_CLIENT_ID,
    "CF-Access-Client-Secret": CF_CLIENT_SECRET,
}
if VAULT_TOKEN:
    HEADERS["X-Vault-Token"] = VAULT_TOKEN

# ポリシーディレクトリの確認
POLICY_DIR = os.path.abspath(os.getenv("POLICY_DIR", "docker/vault/policies"))
if not os.path.isdir(POLICY_DIR):
    raise FileNotFoundError(f"❌ ポリシーディレクトリが見つかりません: {POLICY_DIR}")

# ポリシーごとにVaultへPUT
for filename in os.listdir(POLICY_DIR):
    if not filename.endswith(".hcl"):
        continue

    policy_name = filename.removesuffix(".hcl")
    policy_path = os.path.join(POLICY_DIR, filename)

    with open(policy_path, "r") as f:
        policy_content = f.read()

    url = f"{VAULT_ADDR}/v1/sys/policies/acl/{policy_name}"
    try:
        resp = requests.put(
            url,
            headers=HEADERS,
            json={"policy": policy_content},
            verify=CERT_PATH,
            timeout=15,
        )
        if resp.ok:
            print(f"✅ {policy_name} → アップロード成功")
        else:
            print(f"❌ {policy_name} → {resp.status_code}: {resp.text}")
    except requests.exceptions.RequestException as e:
        print(f"❌ {policy_name} → リクエストエラー: {e}")
