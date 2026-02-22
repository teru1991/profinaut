pub mod config;
pub mod supervisor;

#[cfg(test)]
mod tests {
    use super::config::IngestConfig;
    use super::supervisor::{run_supervisor, should_reconnect_on_stall};

    #[tokio::test(flavor = "current_thread")]
    async fn allowlist_runs_single_exchange() {
        let cfg = IngestConfig {
            exchange_allowlist: Some(vec!["binance".into()]),
            ..Default::default()
        };
        let v = run_supervisor(&cfg).await.unwrap();
        assert_eq!(v, vec!["binance".to_string()]);
    }

    #[test]
    fn stall_detection_requests_reconnect() {
        assert!(should_reconnect_on_stall(61, 60));
        assert!(!should_reconnect_on_stall(59, 60));
    }
}
