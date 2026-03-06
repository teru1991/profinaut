use ucel_core::{AuthMode, AuthRequestMeta};
use ucel_transport::redaction::{redact_sign_input_preview, RedactionPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthRequirements {
    pub requires_auth: bool,
    pub missing_key_id: bool,
    pub missing_auth_mode: bool,
}

pub fn auth_requirements_for_request(meta: &AuthRequestMeta) -> AuthRequirements {
    AuthRequirements {
        requires_auth: meta.requires_auth,
        missing_key_id: meta.requires_auth && meta.key_id.is_none(),
        missing_auth_mode: meta.requires_auth && matches!(meta.auth_mode, AuthMode::None),
    }
}

pub fn preview_redacted_auth_plan(preview: &str) -> String {
    redact_sign_input_preview(&RedactionPolicy::default(), preview)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucel_core::AuthSurface;

    #[test]
    fn requirement_flags_detect_missing_fields() {
        let req = auth_requirements_for_request(&AuthRequestMeta {
            venue: "bitbank".into(),
            surface: AuthSurface::PrivateRest,
            auth_mode: AuthMode::None,
            requires_auth: true,
            request_name: "place_order".into(),
            key_id: None,
        });
        assert!(req.missing_key_id);
        assert!(req.missing_auth_mode);
    }
}
