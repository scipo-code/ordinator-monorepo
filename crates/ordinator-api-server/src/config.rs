use std::str::FromStr;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::anyhow;
use tracing::warn;

use crate::auth::provider::Provider;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment
{
    Production,
    Development,
}

impl FromStr for Environment
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(anyhow!(
                "Invalid environment: '{}'. Must be 'development' or 'production'",
                s
            )),
        }
    }
}

pub struct AppConfig
{
    pub environment: Environment,
    pub dev_bypass_auth: bool,
    pub auth_provider: Provider,
    pub jwt_secret: String,
    pub jwt_token_expiration: i64,
    pub jwt_refresh_token_expiration: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
}

impl AppConfig
{
    pub fn from_env() -> anyhow::Result<Self>
    {
        let environment = dotenvy::var("ENVIRONMENT").context("ENVIRONMENT variable not set")?;
        let environment = Environment::from_str(&environment)?;

        let dev_bypass_auth = dotenvy::var("DEV_BYPASS_AUTH").ok().as_deref() == Some("1");

        if environment == Environment::Production && dev_bypass_auth {
            anyhow::bail!(
                "SECURITY ERROR: DEV_BYPASS_AUTH=1 is set but ENVIRONMENT=production. \
                This is forbidden. Remove DEV_BYPASS_AUTH from production config."
            );
        }

        if dev_bypass_auth {
            warn!("DEV_BYPASS_AUTH is set. This will disable authentication.");
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
            .parse::<i64>()
            .context("JWT_EXPIRATION must be a valid number (seconds)")?;

        let jwt_refresh_expiration = dotenvy::var("REFRESH_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "604800".to_string()) // 7 days default
            .parse::<i64>()
            .context("REFRESH_TOKEN_EXPIRATION must be a valid number (seconds)")?;

        let jwt_issuer =
            dotenvy::var("JWT_ISSUER").context("JWT_ISSUER environment variable must be set")?;

        let jwt_audience =
            dotenvy::var("JWT_ISSUER").context("JWT_ISSUER environment variable must be set")?;

        let server_address =
            dotenvy::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = dotenvy::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid port number")?;

        let database_url = dotenvy::var("DATABASE_URL")
            .context("DATABASE_URL environment variable must be set")?;

        Ok(Self {
            environment,
            dev_bypass_auth,
            auth_provider,
            jwt_secret,
            jwt_token_expiration: jwt_expiration,
            jwt_refresh_token_expiration: jwt_refresh_expiration,
            jwt_issuer,
            jwt_audience,
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
