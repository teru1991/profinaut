use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub http: HttpConfig,
    #[serde(default)]
    pub exchanges: Vec<ExchangeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_listen")]
    pub listen: String,
}

fn default_listen() -> String {
    "127.0.0.1:8087".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub exchange_id: String,
    #[serde(default)]
    pub mode: ExchangeMode,
    #[serde(default)]
    pub params: serde_yaml::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeMode {
    #[default]
    PublicOnly,
    PublicAndPrivate,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid config: exchanges must not be empty")]
    EmptyExchanges,
}

impl AppConfig {
    pub fn load_yaml(path: &std::path::Path) -> Result<Self, ConfigError> {
        let bytes = std::fs::read(path)?;
        let mut cfg: AppConfig = serde_yaml::from_slice(&bytes)?;
        if cfg.exchanges.is_empty() {
            return Err(ConfigError::EmptyExchanges);
        }
        if cfg.http.listen.trim().is_empty() {
            cfg.http.listen = default_listen();
        }
        Ok(cfg)
    }
}
