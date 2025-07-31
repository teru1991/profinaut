import os
import requests
from dotenv import load_dotenv

# 📍 Vaultの自己署名証明書（Cloudflare Origin CA）パス
CA_CERT_PATH = "/vault/cert/origin_ca_rsa_root.pem"

# 🔧 .env ファイル読み込み（Vault Tokenや VAULT_ADDR を取得）
ENV_PATH = "/vault/.env"
if not os.path.exists(ENV_PATH):
    raise FileNotFoundError(f".env ファイルが見つかりません: {ENV_PATH}")

load_dotenv(dotenv_path=ENV_PATH)

VAULT_ADDR = os.getenv("VAULT_ADDR", "https://vault.profinaut.studiokeke.com:8200")
VAULT_TOKEN = os.getenv("VAULT_TOKEN")

if not VAULT_ADDR or not VAULT_TOKEN:
    raise RuntimeError("❌ VAULT_ADDR または VAULT_TOKEN が未定義です。")

HEADERS = {"X-Vault-Token": VAULT_TOKEN}
POLICY_DIR = "/vault/policies"

# 📁 ポリシーファイル存在チェック
if not os.path.exists(POLICY_DIR):
    raise FileNotFoundError(f"ポリシーディレクトリが見つかりません: {POLICY_DIR}")

# 🔁 HCLファイルごとにアップロード処理
for filename in os.listdir(POLICY_DIR):
    if filename.endswith(".hcl"):
        policy_name = filename.replace(".hcl", "")
        policy_path = os.path.join(POLICY_DIR, filename)

        with open(policy_path, "r") as f:
            policy = f.read()

        url = f"{VAULT_ADDR}/v1/sys/policies/acl/{policy_name}"
        data = {"policy": policy}

        try:
            resp = requests.put(url, headers=HEADERS, json=data, verify=CA_CERT_PATH)
            resp.raise_for_status()
            print(f"✅ {policy_name} => アップロード成功（{resp.status_code}）")
        except requests.exceptions.SSLError as e:
            print(f"❌ {policy_name} => SSLエラー: {e}")
        except requests.exceptions.RequestException as e:
            print(f"❌ {policy_name} => リクエストエラー: {e}")
