import os
import yaml
import requests

# 📍 Vault の基本設定
VAULT_ADDR = os.environ.get("VAULT_ADDR", "https://vault.profinaut.studiokeke.com:8200")
VAULT_TOKEN_PATH = "/vault/.vault-token"
CA_CERT_PATH = "/vault/cert/origin_ca_rsa_root.pem"
ROLE_PATH = "auth/approle/role"
HEADERS = {}

# 🔐 Tokenの読み込み
if not os.path.exists(VAULT_TOKEN_PATH):
    raise FileNotFoundError(f"Vault token file not found: {VAULT_TOKEN_PATH}")

with open(VAULT_TOKEN_PATH) as f:
    VAULT_TOKEN = f.read().strip()
    HEADERS = {"X-Vault-Token": VAULT_TOKEN}

# 📜 AppRole定義読み込み
with open("/vault/scripts/../approle_definitions.yaml", "r") as f:
    approle_definitions = yaml.safe_load(f)

# 🛠 AppRoleを順次作成
for role in approle_definitions["approles"]:
    name = role["name"]
    policies = ",".join(role["policies"])

    print(f"\n🔧 AppRole作成中: {name}")

    # Step 1: Create Role
    role_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}"
    role_body = {
        "policies": policies,
        "token_ttl": role["token_ttl"],
        "token_max_ttl": role["token_max_ttl"],
        "secret_id_ttl": role["secret_id_ttl"],
        "secret_id_num_uses": role["secret_id_num_uses"],
    }
    r1 = requests.post(role_url, headers=HEADERS, json=role_body, verify=CA_CERT_PATH)
    r1.raise_for_status()

    # Step 2: Get Role ID
    role_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/role-id"
    role_id_resp = requests.get(role_id_url, headers=HEADERS, verify=CA_CERT_PATH)
    role_id_resp.raise_for_status()
    role_id = role_id_resp.json()["data"]["role_id"]

    # Step 3: Generate Secret ID
    secret_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/secret-id"
    secret_id_resp = requests.post(secret_id_url, headers=HEADERS, verify=CA_CERT_PATH)
    secret_id_resp.raise_for_status()
    secret_id = secret_id_resp.json()["data"]["secret_id"]

    print(f"✅ AppRole: {name}")
    print(f"  Role ID   : {role_id}")
    print(f"  Secret ID : {secret_id}")

# 💾 環境変数出力ファイル生成
output_path = "/vault/scripts/env.generated"
with open(output_path, "w") as f:
    for role in approle_definitions["approles"]:
        name = role["name"]
        env_var = name.upper().replace("-", "_")

        # 再取得（安全のため）
        role_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/role-id"
        role_id = requests.get(
            role_id_url, headers=HEADERS, verify=CA_CERT_PATH
        ).json()["data"]["role_id"]

        secret_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/secret-id"
        secret_id = requests.post(
            secret_id_url, headers=HEADERS, verify=CA_CERT_PATH
        ).json()["data"]["secret_id"]

        f.write(f"{env_var}_ROLE_ID={role_id}\n")
        f.write(f"{env_var}_SECRET_ID={secret_id}\n")

print(f"\n📝 .env.generated が出力されました: {output_path}")
