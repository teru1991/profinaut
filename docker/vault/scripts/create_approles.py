import os
import yaml
import requests

# Vault Token をファイルから読む（/vault/.vault-token に保存される）
with open("/vault/.vault-token") as f:
    VAULT_TOKEN = f.read().strip()

VAULT_ADDR = os.environ.get("VAULT_ADDR", "https://vault:8200")
CA_CERT_PATH = "/vault/cert/origin_ca_rsa_root.pem"
HEADERS = {"X-Vault-Token": VAULT_TOKEN}

ROLE_PATH = "auth/approle/role"

# AppRole定義をロード
with open("../approle_definitions.yaml", "r") as f:
    approle_definitions = yaml.safe_load(f)

for role in approle_definitions["approles"]:
    name = role["name"]
    policies = ",".join(role["policies"])

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

    # Step 2: Get Role ID
    role_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/role-id"
    role_id = requests.get(role_id_url, headers=HEADERS, verify=CA_CERT_PATH).json()["data"]["role_id"]

    # Step 3: Generate Secret ID
    secret_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/secret-id"
    secret_id = requests.post(secret_id_url, headers=HEADERS, verify=CA_CERT_PATH).json()["data"]["secret_id"]

    print(f"\n✅ AppRole: {name}")
    print(f"  Role ID: {role_id}")
    print(f"  Secret ID: {secret_id}")

# 出力ファイル生成
with open("env.generated", "w") as f:
    for role in approle_definitions["approles"]:
        name = role["name"]

        # role_id 取得
        role_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/role-id"
        role_id = requests.get(role_id_url, headers=HEADERS, verify=CA_CERT_PATH).json()["data"]["role_id"]

        # secret_id 取得
        secret_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/secret-id"
        secret_id = requests.post(secret_id_url, headers=HEADERS, verify=CA_CERT_PATH).json()["data"]["secret_id"]

        # 変数名用に整形
        env_var_name = name.upper().replace("-", "_")
        f.write(f"{env_var_name}_ROLE_ID={role_id}\n")
        f.write(f"{env_var_name}_SECRET_ID={secret_id}\n")