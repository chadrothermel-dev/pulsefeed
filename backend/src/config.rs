use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub db_path: String,
    pub jwt_secret: String,
    pub static_dir: String,
    pub max_concurrent_fetches: usize,
    pub poll_interval_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("PULSEFEED_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PULSEFEED_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            db_path: env::var("PULSEFEED_DB").unwrap_or_else(|_| "pulsefeed.db".to_string()),
            jwt_secret: env::var("PULSEFEED_JWT_SECRET")
                .unwrap_or_else(|_| "pulsefeed-dev-secret-change-in-production-9f8a7b6c".to_string()),
            static_dir: env::var("PULSEFEED_STATIC_DIR")
                .unwrap_or_else(|_| "../frontend/dist".to_string()),
            max_concurrent_fetches: env::var("PULSEFEED_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            poll_interval_secs: env::var("PULSEFEED_POLL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
        }
    }
}
