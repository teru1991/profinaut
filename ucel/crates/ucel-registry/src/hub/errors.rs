use thiserror::Error;

#[derive(Debug, Error)]
pub enum HubError {
    #[error("unknown exchange: {0}")]
    UnknownExchange(String),
    #[error("unknown operation key: {exchange}:{key}")]
    UnknownOperation { exchange: String, key: String },
    #[error("unknown channel key: {exchange}:{key}")]
    UnknownChannel { exchange: String, key: String },
    #[error("registry validation failed: {0}")]
    RegistryValidation(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("ws error: {0}")]
    Ws(#[source] Box<tokio_tungstenite::tungstenite::Error>),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<tokio_tungstenite::tungstenite::Error> for HubError {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(Box::new(value))
    }
}
