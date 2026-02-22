pub mod config;
pub mod supervisor;

#[cfg(test)]
mod tests {
    use super::config::IngestConfig;
    use super::supervisor::run_supervisor;

    #[tokio::test(flavor = "current_thread")]
    async fn allowlist_runs_single_exchange() {
        let cfg = IngestConfig {
            exchange_allowlist: Some(vec!["binance".into()]),
            store_path: ":memory:".into(),
            ..Default::default()
        };
        let v = run_supervisor(&cfg).await.unwrap();
        assert_eq!(v, vec!["binance".to_string()]);
    }
}
