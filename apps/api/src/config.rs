use std::{
    env,
    net::{IpAddr, SocketAddr},
    num::ParseIntError,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_host: IpAddr,
    pub api_port: u16,
    pub cors_allowed_origin: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?;
        let api_host = env::var("API_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned())
            .parse()
            .map_err(ConfigError::InvalidHost)?;
        let api_port = env::var("API_PORT")
            .unwrap_or_else(|_| "4000".to_owned())
            .parse()
            .map_err(ConfigError::InvalidPort)?;
        let cors_allowed_origin = env::var("CORS_ALLOWED_ORIGIN")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());

        Ok(Self {
            database_url,
            api_host,
            api_port,
            cors_allowed_origin,
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from((self.api_host, self.api_port))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    MissingEnv(&'static str),
    #[error("API_HOST is not a valid IP address")]
    InvalidHost(std::net::AddrParseError),
    #[error("API_PORT is not a valid port")]
    InvalidPort(ParseIntError),
}

