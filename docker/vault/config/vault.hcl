listener "tcp" {
  address         = "0.0.0.0:8200"
  tls_disable     = false
  tls_cert_file   = "/vault/cert/vault-cert.pem"
  tls_key_file    = "/vault/cert/vault-key.pem"
  tls_min_version = "tls12"
}

# 💾 ストレージ設定（fileベースの永続化）
storage "file" {
  path = "/vault/data"
}

# 🔐 Auto-Unseal に AWS KMS を使用
seal "awskms" {
  region     = "ap-northeast-1"
  kms_key_id = "7b9073cd-c31b-46ee-89c5-0f3fe28c8887"
}

# 🌐 UIとログ設定
ui = true
log_level = "info"

# 🔗 Cloudflare経由で公開されるFQDNを指定
api_addr     = "https://vault.profinaut.studiokeke.com:8200"
cluster_addr = "https://vault.profinaut.studiokeke.com:8200"


