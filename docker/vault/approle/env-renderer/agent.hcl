pid_file = "/agent/pidfile"

vault {
  address = "https://vault:8200"
  ca_cert = "/etc/ssl/private/vault_root_ca.pem"
}

auto_auth {
  method "approle" {
    mount_path = "auth/approle"
    config = {
      role_id_file_path   = "/approle/role_id"
      secret_id_file_path = "/approle/secret_id"
    }
  }

  sink "file" {
    config = { path = "/agent/.token" }
  }
}

cache { use_auto_auth_token = true }

template {
  source      = "/agent/templates/master_env.tpl"
  destination = "/agent/out/master.env"
  perms       = "0600"
}
