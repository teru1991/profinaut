use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WsHandle {
    pub send_tx: mpsc::Sender<String>,
    pub recv_rx: mpsc::Receiver<Vec<u8>>,
}

pub async fn connect(url: &str) -> Result<(), String> {
    let _ = connect_async(url).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Open a WebSocket connection and return a [`WsHandle`] for send/recv.
/// Spawns internal tasks to drive the connection.
pub async fn open(
    url: &str,
    send_capacity: usize,
    recv_capacity: usize,
) -> Result<WsHandle, String> {
    let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (send_tx, mut send_rx) = mpsc::channel::<String>(send_capacity);
    let (recv_tx, recv_rx) = mpsc::channel::<Vec<u8>>(recv_capacity);

    let url_owned = url.to_string();

    // Writer task: forward outbound messages to the WS sink.
    tokio::spawn(async move {
        while let Some(msg) = send_rx.recv().await {
            if let Err(e) = ws_write.send(Message::Text(msg.into())).await {
                tracing::warn!(url = %url_owned, "WS send error: {e}");
                break;
            }
        }
    });

    let url_owned2 = url.to_string();

    // Reader task: forward inbound frames to the recv channel.
    tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(txt)) => {
                    let _ = recv_tx.send(txt.as_bytes().to_vec()).await;
                }
                Ok(Message::Binary(bin)) => {
                    let _ = recv_tx.send(bin.into()).await;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Ok(Message::Close(frame)) => {
                    tracing::debug!(url = %url_owned2, "WS closed: {:?}", frame);
                    break;
                }
                Err(e) => {
                    tracing::warn!(url = %url_owned2, "WS read error: {e}");
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(WsHandle { send_tx, recv_rx })
}
