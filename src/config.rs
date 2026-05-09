use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TenantConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub tenants: Vec<TenantConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            tenants: vec![TenantConfig::default()],
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_db_url(),
            max_connections: default_max_connections(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            id: "default".into(),
            name: "Default Tenant".into(),
            schema: default_schema(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_port() -> u16 {
    8080
}

fn default_db_url() -> String {
    "postgres://localhost:5432/fb".into()
}

fn default_max_connections() -> u32 {
    5
}

fn default_log_level() -> String {
    "info".into()
}

fn default_schema() -> String {
    "public".into()
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        let opts = PgConnectOptions::from_str(&self.url)?;
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect_with(opts)
            .await
    }
}
