use tokio_tungstenite::connect_async;

pub async fn connect(url: &str) -> Result<(), String> {
    let _ = connect_async(url).await.map_err(|e| e.to_string())?;
    Ok(())
}
