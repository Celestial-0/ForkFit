use std::{env, net::SocketAddr, time::Duration};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub redis_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub admin_email: Option<String>,
    pub session_ttl: Duration,
    pub otp_ttl: Duration,
    pub otp_max_attempts: i32,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing DATABASE_URL or POSTGRES_DB/POSTGRES_USER/POSTGRES_PASSWORD environment")]
    MissingDatabase,
    #[error("invalid API_BIND_ADDR: {0}")]
    InvalidBindAddr(#[from] std::net::AddrParseError),
    #[error("missing OAuth configuration: {0}")]
    MissingOauth(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env::var("API_BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3001".to_string())
            .parse()?;

        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_password = env::var("REDIS_PASSWORD").ok().filter(|s| !s.trim().is_empty());

        let redis_url = if let Some(pass) = redis_password {
            format!("redis://:{}@{}:{}", pass, redis_host, redis_port)
        } else {
            format!("redis://{}:{}", redis_host, redis_port)
        };

        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| ConfigError::MissingOauth("GOOGLE_CLIENT_ID is required".to_string()))?;
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| ConfigError::MissingOauth("GOOGLE_CLIENT_SECRET is required".to_string()))?;
        let github_client_id = env::var("GITHUB_CLIENT_ID")
            .map_err(|_| ConfigError::MissingOauth("GITHUB_CLIENT_ID is required".to_string()))?;
        let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| ConfigError::MissingOauth("GITHUB_CLIENT_SECRET is required".to_string()))?;

        Ok(Self {
            bind_addr,
            database_url: database_url_from_env()?,
            redis_url,
            google_client_id,
            google_client_secret,
            github_client_id,
            github_client_secret,
            admin_email: env::var("ADMIN_EMAIL")
                .ok()
                .map(|email| email.trim().to_lowercase())
                .filter(|email| !email.is_empty()),
            session_ttl: Duration::from_secs(env_u64("SESSION_TTL_SECONDS", 60 * 60 * 24 * 30)),
            otp_ttl: Duration::from_secs(env_u64("OTP_TTL_SECONDS", 60 * 10)),
            otp_max_attempts: env_i32("OTP_MAX_ATTEMPTS", 5),
            smtp_host: env::var("SMTP_HOST").ok().filter(|s| !s.trim().is_empty()),
            smtp_port: env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()),
            smtp_user: env::var("SMTP_USER").ok().filter(|s| !s.trim().is_empty()),
            smtp_password: env::var("SMTP_PASSWORD").ok().filter(|s| !s.trim().is_empty()),
            smtp_from: env::var("SMTP_FROM").ok().filter(|s| !s.trim().is_empty()),
        })
    }
}

fn database_url_from_env() -> Result<String, ConfigError> {
    if let Ok(url) = env::var("DATABASE_URL")
        && !url.trim().is_empty()
    {
        return Ok(url);
    }

    let db = env::var("POSTGRES_DB").map_err(|_| ConfigError::MissingDatabase)?;
    let user = env::var("POSTGRES_USER").map_err(|_| ConfigError::MissingDatabase)?;
    let password = env::var("POSTGRES_PASSWORD").map_err(|_| ConfigError::MissingDatabase)?;
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());

    Ok(format!("postgres://{user}:{password}@{host}:{port}/{db}"))
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_i32(name: &str, default: i32) -> i32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
