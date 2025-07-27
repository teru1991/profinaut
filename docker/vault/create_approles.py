# create_approles.py
import os
import yaml
import requests
from dotenv import load_dotenv

load_dotenv(dotenv_path="../../.env")

VAULT_ADDR = os.getenv("VAULT_ADDR")
VAULT_TOKEN = os.getenv("VAULT_TOKEN")

HEADERS = {"X-Vault-Token": VAULT_TOKEN}
ROLE_PATH = "auth/approle/role"

with open("approle_definitions.yaml", "r") as f:
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
    r1 = requests.post(role_url, headers=HEADERS, json=role_body)

    # Step 2: Get Role ID
    role_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/role-id"
    role_id = requests.get(role_id_url, headers=HEADERS).json()["data"]["role_id"]

    # Step 3: Generate Secret ID
    secret_id_url = f"{VAULT_ADDR}/v1/{ROLE_PATH}/{name}/secret-id"
    secret_id = requests.post(secret_id_url, headers=HEADERS).json()["data"][
        "secret_id"
    ]

    print(f"\n✅ AppRole: {name}")
    print(f"  Role ID: {role_id}")
    print(f"  Secret ID: {secret_id}")
