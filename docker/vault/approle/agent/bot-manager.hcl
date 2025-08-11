exit_on_retry_failure = true

auto_auth {
  method "approle" {
    mount_path = "auth/approle"
    config = {
      role_id_file_path   = "/approle/roles/bot_manager_role_id"
      secret_id_file_path = "/approle/roles/bot_manager_secret_id"
    }
  }
  sink "file" {
    config = {
      path = "/env.generated/.agent_token"
    }
  }
}

template {
  source      = "/approle/templates/bot_manager.env.tmpl"
  destination = "/env.generated/bot_manager.env"
  command     = "chmod 600 /env.generated/bot_manager.env"
}
