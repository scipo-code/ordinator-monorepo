use std::str::FromStr;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::anyhow;
use tracing::warn;

use crate::auth::provider::Provider;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub struct AppConfig
{
    pub auth_provider: Provider,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub refresh_expiration: u64,
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
}

impl AppConfig
{
    pub fn from_env() -> anyhow::Result<Self>
    {
        if let Ok(bypass_auth) = dotenvy::var("DEV_BYPASS_AUTH") {
            if bypass_auth == "1" {
                warn!("DEV_BYPASS_AUTH is set. This will disable authentication.");
            } else {
                warn!("DEV_BYPASS_AUTH is defined but not initialized");
            }
        }
        let auth_provider = dotenvy::var("AUTH_PROVIDER")
            .context("AUTH_PROVIDER environment variable must be set")?;

        let auth_provider = Provider::from_str(&auth_provider)
            .map_err(|e| anyhow::anyhow!("Invalid AUTH_PROVIDER '{}': {}", auth_provider, e))?;

        let jwt_secret =
            dotenvy::var("JWT_SECRET").context("JWT_SECRET environment variable must be set")?;

        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }

        let jwt_expiration = dotenvy::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "3600".to_string()) // 1 hour default
            .parse::<u64>()
            .context("JWT_EXPIRATION must be a valid number (seconds)")?;

        let refresh_expiration = dotenvy::var("REFRESH_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "604800".to_string()) // 7 days default
            .parse::<u64>()
            .context("REFRESH_TOKEN_EXPIRATION must be a valid number (seconds)")?;

        let server_address =
            dotenvy::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = dotenvy::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid port number")?;

        let database_url = dotenvy::var("DATABASE_URL")
            .context("DATABASE_URL environment variable must be set")?;

        Ok(Self {
            auth_provider,
            jwt_secret,
            jwt_expiration,
            refresh_expiration,
            server_address,
            server_port,
            database_url,
        })
    }
}

pub fn init_config() -> anyhow::Result<&'static AppConfig>
{
    let config = AppConfig::from_env()?;
    CONFIG
        .set(config)
        .map_err(|_| anyhow!("Config already initialized"))?;
    Ok(CONFIG.get().unwrap())
}

pub fn get_config() -> &'static AppConfig
{
    CONFIG
        .get()
        .expect("Config not initialized - call init_config first")
}
