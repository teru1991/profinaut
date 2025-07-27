# create_policies.py
import os
import requests
from dotenv import load_dotenv

load_dotenv(dotenv_path="../../.env")

VAULT_ADDR = os.getenv("VAULT_ADDR")
VAULT_TOKEN = os.getenv("VAULT_TOKEN")  # 実行時のみ手動入力 or 一時的に .env に追加

POLICY_DIR = "./policies"

headers = {"X-Vault-Token": VAULT_TOKEN}

for filename in os.listdir(POLICY_DIR):
    if filename.endswith(".hcl"):
        policy_name = filename.replace(".hcl", "")
        with open(os.path.join(POLICY_DIR, filename), "r") as f:
            policy = f.read()

        url = f"{VAULT_ADDR}/v1/sys/policies/acl/{policy_name}"
        data = {"policy": policy}
        resp = requests.put(url, headers=headers, json=data)

        print(f"[{policy_name}] => Status: {resp.status_code}, Message: {resp.text}")
