use crate::health::Health;

pub fn handle_rest_failure(health: &mut Health) {
    health.mark_reason("rest_failed");
}

pub fn handle_rest_success(health: &mut Health) {
    health.clear_reason("rest_failed");
}
