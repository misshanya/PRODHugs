use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub http: HttpServer,
    #[serde(default)]
    pub metrics: MetricsServer,
    pub postgres: Postgres,
    #[serde(default)]
    pub cors: Cors,
    pub jwt: Jwt,
    #[serde(default)]
    pub telegram: Telegram,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpServer {
    pub addr: String,
}

impl Default for HttpServer {
    fn default() -> Self {
        Self {
            addr: ":8080".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsServer {
    pub addr: String,
}

impl Default for MetricsServer {
    fn default() -> Self {
        Self {
            addr: ":9090".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Postgres {
    pub url: String,
    #[serde(default = "default_max_conns")]
    pub max_conns: i32,
    #[serde(default = "default_min_conns")]
    pub min_conns: i32,
    #[serde(default = "default_max_conn_lifetime")]
    pub max_conn_lifetime: i64,
}

fn default_max_conns() -> i32 {
    100
}
fn default_min_conns() -> i32 {
    5
}
fn default_max_conn_lifetime() -> i64 {
    3600
}

#[derive(Clone, Debug, Deserialize)]
pub struct Cors {
    #[serde(default = "default_cors")]
    pub allow_origins: Vec<String>,
}

impl Default for Cors {
    fn default() -> Self {
        Self {
            allow_origins: default_cors(),
        }
    }
}

fn default_cors() -> Vec<String> {
    vec![
        "http://localhost:3000".into(),
        "http://localhost:3001".into(),
    ]
}

#[derive(Clone, Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
    #[serde(default = "default_access")]
    pub access_duration: i64,
    #[serde(default = "default_refresh")]
    pub refresh_duration: i64,
    #[serde(default = "default_cookie_secure")]
    pub cookie_secure: bool,
}

fn default_access() -> i64 {
    900
}
fn default_refresh() -> i64 {
    604_800
}
fn default_cookie_secure() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Telegram {
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub bot_username: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let cfg = Self {
            http: HttpServer {
                addr: env_or("SERVER_ADDR", ":8080"),
            },
            metrics: MetricsServer {
                addr: env_or("METRICS_ADDR", ":9090"),
            },
            postgres: Postgres {
                url: std::env::var("POSTGRES_URL")
                    .map_err(|_| anyhow!("POSTGRES_URL is required"))?,
                max_conns: env_i32("POSTGRES_MAX_CONNS", 100)?,
                min_conns: env_i32("POSTGRES_MIN_CONNS", 5)?,
                max_conn_lifetime: env_i64("POSTGRES_MAX_CONN_LIFETIME", 3600)?,
            },
            cors: Cors {
                allow_origins: env_list("CORS_ALLOW_ORIGINS", default_cors()),
            },
            jwt: Jwt {
                secret: std::env::var("JWT_SECRET")
                    .map_err(|_| anyhow!("JWT_SECRET is required"))?,
                access_duration: env_i64("JWT_ACCESS_DURATION", 900)?,
                refresh_duration: env_i64("JWT_REFRESH_DURATION", 604_800)?,
                cookie_secure: env_bool("JWT_COOKIE_SECURE", true),
            },
            telegram: Telegram {
                bot_token: env_or("TELEGRAM_BOT_TOKEN", ""),
                bot_username: env_or("TELEGRAM_BOT_USERNAME", ""),
            },
        };
        Ok(cfg)
    }

    pub fn validate_security(&self) -> Result<()> {
        let secret = self.jwt.secret.trim();
        if secret.len() < 32 {
            return Err(anyhow!("JWT_SECRET must be at least 32 characters"));
        }
        let lower = secret.to_ascii_lowercase();
        const WEAK: &[&str] = &[
            "change-me",
            "changeme",
            "jwt-secret",
            "secret",
            "hugs-as-a-service-super-secret-jwt-key-2026",
        ];
        if WEAK.contains(&lower.as_str()) {
            return Err(anyhow!("JWT_SECRET uses a known weak/default value"));
        }
        Ok(())
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_i32(key: &str, default: i32) -> Result<i32> {
    match std::env::var(key) {
        Ok(v) => Ok(v.parse()?),
        Err(_) => Ok(default),
    }
}

fn env_i64(key: &str, default: i64) -> Result<i64> {
    match std::env::var(key) {
        Ok(v) => Ok(v.parse()?),
        Err(_) => Ok(default),
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_list(key: &str, default: Vec<String>) -> Vec<String> {
    match std::env::var(key) {
        Ok(v) => v
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect(),
        Err(_) => default,
    }
}
