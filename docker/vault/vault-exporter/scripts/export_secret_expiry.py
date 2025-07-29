import os
import requests
from flask import Flask, Response

app = Flask(__name__)

VAULT_ADDR = os.getenv("VAULT_ADDR", "http://localhost:8200")
ROLE_ID = os.getenv("VAULT_ROLE_ID")
SECRET_ID = os.getenv("VAULT_SECRET_ID")
ROLE_NAME = os.getenv("VAULT_ROLE_NAME", "profinaut_admin")


@app.route("/metrics")
def metrics():
    try:
        # Vaultにログイン
        login_resp = requests.post(
            f"{VAULT_ADDR}/v1/auth/approle/login",
            json={"role_id": ROLE_ID, "secret_id": SECRET_ID},
        )
        login_resp.raise_for_status()
        client_token = login_resp.json()["auth"]["client_token"]

        # トークンの TTL を取得
        headers = {"X-Vault-Token": client_token}
        lookup_resp = requests.get(
            f"{VAULT_ADDR}/v1/auth/token/lookup-self", headers=headers
        )
        lookup_resp.raise_for_status()
        ttl_seconds = lookup_resp.json()["data"]["ttl"]

        # Prometheusメトリクス出力
        metrics_output = (
            f'vault_secret_expiry_seconds{{role="{ROLE_NAME}"}} {ttl_seconds}\n'
        )
        return Response(metrics_output, mimetype="text/plain")

    except Exception as e:
        return Response(
            f'vault_secret_exporter_error{{role="{ROLE_NAME}"}} 1\n# Error: {str(e)}\n',
            mimetype="text/plain",
        )


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=9811)
