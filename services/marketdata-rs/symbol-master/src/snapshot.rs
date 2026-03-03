use serde_json::Value;

#[derive(thiserror::Error, Debug)]
pub enum SnapshotError {
    #[error("snapshot_url missing")]
    MissingUrl,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct RawSnapshot {
    pub exchange_id: String,
    pub body: Value,
}

pub async fn fetch_snapshot(
    exchange_id: &str,
    snapshot_url: &str,
) -> Result<RawSnapshot, SnapshotError> {
    if snapshot_url.trim().is_empty() {
        return Err(SnapshotError::MissingUrl);
    }
    let client = reqwest::Client::builder().no_proxy().build()?;
    let body: Value = client
        .get(snapshot_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(RawSnapshot {
        exchange_id: exchange_id.to_string(),
        body,
    })
}
