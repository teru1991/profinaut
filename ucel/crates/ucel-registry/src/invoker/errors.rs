use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvokerError {
    #[error("invalid venue id: {0}")]
    InvalidVenueId(String),
    #[error("invalid operation id: {0}")]
    InvalidOperationId(String),
    #[error("invalid market symbol: {0}")]
    InvalidMarketSymbol(String),
    #[error("unknown venue: {0}")]
    UnknownVenue(String),
    #[error("unknown operation id: {venue}:{id}")]
    UnknownOperation { venue: String, id: String },
    #[error("kind mismatch for {venue}:{id}, expected={expected}, actual={actual}")]
    KindMismatch {
        venue: String,
        id: String,
        expected: String,
        actual: String,
    },
    #[error("registry validation failed: {0}")]
    RegistryValidation(String),
    #[error("missing placeholder values in template: {0}")]
    MissingPlaceholder(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("ws error: {0}")]
    Ws(#[source] Box<tokio_tungstenite::tungstenite::Error>),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<tokio_tungstenite::tungstenite::Error> for InvokerError {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(Box::new(value))
    }
}
