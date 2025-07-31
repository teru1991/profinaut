import os
import requests

# ✅ Vault アドレスと証明書パス
VAULT_ADDR = os.getenv("VAULT_ADDR", "https://vault.profinaut.studiokeke.com:8200")
CERT_PATH = os.getenv("VAULT_CERT", "/vault/cert/origin_ca_rsa_root.pem")

# ✅ Cloudflare Access Service Token
CF_CLIENT_ID = os.getenv("CF_ACCESS_CLIENT_ID")
CF_CLIENT_SECRET = os.getenv("CF_ACCESS_CLIENT_SECRET")

# ✅ オプションの Vault Token（AppRoleまたはOIDCで取得する場合）
VAULT_TOKEN = os.getenv("VAULT_TOKEN")

# ✅ ヘッダー構築（Cloudflare Access優先）
HEADERS = {
    "CF-Access-Client-Id": CF_CLIENT_ID,
    "CF-Access-Client-Secret": CF_CLIENT_SECRET,
}
if VAULT_TOKEN:
    HEADERS["X-Vault-Token"] = VAULT_TOKEN

# ✅ ポリシーディレクトリ（デフォルト: docker/vault/policies）
POLICY_DIR = os.path.abspath(os.getenv("POLICY_DIR", "docker/vault/policies"))
if not os.path.exists(POLICY_DIR):
    raise FileNotFoundError(f"ポリシーディレクトリが見つかりません: {POLICY_DIR}")

# ✅ ポリシーファイルをループ処理でアップロード
for filename in os.listdir(POLICY_DIR):
    if filename.endswith(".hcl"):
        policy_name = filename.replace(".hcl", "")
        policy_path = os.path.join(POLICY_DIR, filename)

        with open(policy_path, "r") as f:
            policy_content = f.read()

        url = f"{VAULT_ADDR}/v1/sys/policies/acl/{policy_name}"
        response = requests.put(
            url,
            headers=HEADERS,
            json={"policy": policy_content},
            verify=CERT_PATH,
        )

        if response.ok:
            print(f"✅ {policy_name} => アップロード成功（{response.status_code}）")
        else:
            print(f"❌ {policy_name} => エラー（{response.status_code}）: {response.text}")