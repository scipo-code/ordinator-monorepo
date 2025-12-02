pub mod local;

use std::fmt::Display;
use std::str::FromStr;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::models::AuthError;
use crate::auth::models::TokenClaims;

// For local - username/password authentication
#[async_trait]
pub trait CredentialAuthProvider
{
    type Credentials;

    async fn authenticate(&self, credentials: Self::Credentials) -> Result<TokenClaims, AuthError>;
}

#[async_trait]
pub trait TokenValidator
{
    async fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthError>;
}

// For OAuth providers like Azure - token based
#[async_trait]
pub trait OAuthProvider: TokenValidator
{
    fn get_authorization_url(&self) -> String;
    async fn exhange_code(&self, code: String) -> Result<String, AuthError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Provider
{
    Local,
    Azure,
}

impl FromStr for Provider
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s {
            "Local" => Ok(Provider::Local),
            "Azure" => Ok(Provider::Azure),
            _ => Err(format!("Invalid Provider: {}", s)),
        }
    }
}

impl Display for Provider
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Provider::Local => write!(f, "Local"),
            Provider::Azure => write!(f, "Azure"),
        }
    }
}
