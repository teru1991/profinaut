import os
import requests
from dotenv import load_dotenv

# Vaultの自己署名証明書のパス
CA_CERT_PATH = "/vault/cert/origin_ca_rsa_root.pem"

# .envファイルから環境変数読み込み
load_dotenv(dotenv_path="/vault/.env")

VAULT_ADDR = os.getenv("VAULT_ADDR")
VAULT_TOKEN = os.getenv("VAULT_TOKEN")  # 起動時に一時設定される前提

if not VAULT_ADDR or not VAULT_TOKEN:
    raise RuntimeError("VAULT_ADDR または VAULT_TOKEN が未定義です。")

POLICY_DIR = "/vault/policies"  # Dockerコンテナ内でマウントされるパス
HEADERS = {"X-Vault-Token": VAULT_TOKEN}

# HCLファイルをすべて処理
for filename in os.listdir(POLICY_DIR):
    if filename.endswith(".hcl"):
        policy_name = filename.replace(".hcl", "")
        with open(os.path.join(POLICY_DIR, filename), "r") as f:
            policy = f.read()

        url = f"{VAULT_ADDR}/v1/sys/policies/acl/{policy_name}"
        data = {"policy": policy}
        try:
            resp = requests.put(url, headers=HEADERS, json=data, verify=CA_CERT_PATH)
            print(f"[{policy_name}] => Status: {resp.status_code}, Message: {resp.text}")
        except requests.exceptions.SSLError as e:
            print(f"[{policy_name}] => SSLエラー: {str(e)}")
        except Exception as e:
            print(f"[{policy_name}] => エラー: {str(e)}")