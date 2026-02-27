use thiserror::Error;

pub type SdkResult<T> = Result<T, SdkError>;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("config invalid: {0}")]
    Config(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("hub: {0}")]
    Hub(#[from] ucel_registry::hub::HubError),

    #[error("unknown exchange: {0}")]
    UnknownExchange(String),
}
