use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

use argon2::Argon2;
use argon2::password_hash::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use axum::Json;
use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::current_timestamp;
use crate::auth::jwt::JWT_CONFIG;
use crate::auth::provider::Provider;

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = JWT_CONFIG.secret.clone();

    Keys::new(secret.as_bytes())
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims
{
    pub sub: String,
    // The JWT ID: Unique Identifier for token. Should map to token_uuid in
    // Token Details. Here kept as jti for mapping to standard practice.
    pub jti: String,
    // Expires at. How do we set the valid duration lifetime of the JWT?
    // Right now hardcoded as 1 hr.
    pub exp: i64,
    pub iat: i64,
    // Not before: Means when is this token valid.
    pub nbf: i64,
    pub iss: String,
    pub aud: String,

    // Custome fields
    pub role: UserRole,
    pub assets: Vec<String>,
    pub provider: Provider,
}

impl TokenClaims
{
    pub fn new(sub: String, role: UserRole, assets: Vec<String>, provider: Provider) -> Self
    {
        let now = current_timestamp();
        Self {
            sub,
            jti: Uuid::new_v4().to_string(),
            exp: now + JWT_CONFIG.access_token_expiration,
            iat: now,
            nbf: now,
            iss: JWT_CONFIG.issuer.clone(),
            aud: JWT_CONFIG.audience.clone(),

            role,
            assets,
            provider,
        }
    }

    pub fn require_asset(&self, asset: &str) -> Result<(), AuthError>
    {
        if self.assets.contains(&asset.to_string()) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }

    // WARN This is a dev function for bypassing authentication
    // This should only EVER be used in dev using DEB_BYPASS_AUTH=1
    pub fn dev_default(role: UserRole) -> Self
    {
        TokenClaims::new(
            "test@ordinator.io".to_string(),
            role,
            vec!["test".to_string()],
            Provider::Local,
        )
    }
}

impl Display for TokenClaims
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "subject: {}\nrole: {:?}\nassets: {:?}",
            self.sub, self.role, self.assets
        )
    }
}

impl<S> FromRequestParts<S> for TokenClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data =
            decode::<TokenClaims>(bearer.token(), &KEYS.decoding, &Validation::default())
                .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenClaims
{
    pub sub: String,
    // The JWT ID: Unique Identifier for token. Should map to token_uuid in
    // Token Details. Here kept as jti for mapping to standard practice.
    pub jti: String,
    // Expires at. How do we set the valid duration lifetime of the JWT?
    // Right now hardcoded as 1 hr.
    pub exp: i64,
    pub iat: i64,
    // Not before: Means when is this token valid.
    pub nbf: i64,
    pub iss: String,
    pub aud: String,

    pub provider: Provider,
    pub token_type: String,
}

impl RefreshTokenClaims
{
    pub fn new(sub: String, provider: Provider) -> Self
    {
        let now = current_timestamp();
        Self {
            sub,
            jti: Uuid::new_v4().to_string(),
            exp: now + JWT_CONFIG.refresh_token_expiration,
            iat: now,
            nbf: now,
            iss: JWT_CONFIG.issuer.clone(),
            aud: JWT_CONFIG.audience.clone(),
            provider,
            token_type: "refresh".to_string(),
        }
    }
}

impl<S> FromRequestParts<S> for RefreshTokenClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data =
            decode::<RefreshTokenClaims>(bearer.token(), &KEYS.decoding, &Validation::default())
                .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody
{
    access_token: String,
    token_type: String,
}

impl AuthBody
{
    pub fn new(access_token: String) -> Self
    {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

pub struct Keys
{
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys
{
    pub fn new(secret: &[u8]) -> Self
    {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, ToSchema)]
pub enum AuthError
{
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    TokenExpired,
    UserNotFound,
    UserInactive,
    InternalError,
    Forbidden,
}

impl IntoResponse for AuthError
{
    fn into_response(self) -> axum::response::Response
    {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::TokenExpired => (StatusCode::BAD_REQUEST, "Token expired"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::UserInactive => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Server error"),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
        };
        let body = Json(json!(
            {"error": error_message}
        ));

        (status, body).into_response()
    }
}

//////////////  User Models //////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole
{
    Scheduler,
    Supervisor,
    Technician,
    Admin,
}
impl FromStr for UserRole
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s {
            "Scheduler" => Ok(UserRole::Scheduler),
            "Supervisor" => Ok(UserRole::Supervisor),
            "Technician" => Ok(UserRole::Technician),
            "Admin" => Ok(UserRole::Admin),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

impl std::fmt::Display for UserRole
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            UserRole::Scheduler => write!(f, "Scheduler"),
            UserRole::Supervisor => write!(f, "Supervisor"),
            UserRole::Technician => write!(f, "Technician"),
            UserRole::Admin => write!(f, "Admin"),
        }
    }
}

pub type UserId = Uuid;
#[derive(Clone)]
pub struct User
{
    pub id: UserId,
    pub email: String,
    pub password_hash: Option<String>, //Argon2 with salt string embedded
    pub role: UserRole,
    pub assets: Vec<String>,
    pub provider: Provider,
    pub is_active: bool,
}

impl User
{
    pub fn new(
        id: UserId,
        email: String,
        password_str: Option<String>,
        role: UserRole,
        assets: Vec<String>,
        provider: Provider,
        is_active: bool,
    ) -> Result<Self, UserError>
    {
        // The password should here be validated for length,
        // special characters etc.
        let pwd_hash = match provider {
            Provider::Local => {
                let pwd = password_str.ok_or_else(|| UserError::MissingPassword)?;
                let argon2 = Argon2::default();
                let salt = SaltString::generate(&mut OsRng);

                // Hash password to PHC string ($argon2id$v=19$...)
                let pwd_hash = argon2
                    .hash_password(pwd.as_bytes(), &salt)
                    .map_err(|e| UserError::HashError(e))?
                    .to_string();
                Some(pwd_hash)
            }
            _ => None,
        };

        Ok(Self {
            id,
            email,
            password_hash: pwd_hash,
            role,
            assets,
            provider: Provider::Local,
            is_active,
        })
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, UserError>
    {
        let password_hash_str = self
            .password_hash
            .as_ref()
            .ok_or_else(|| UserError::MissingPassword)?;
        let parsed_hash =
            PasswordHash::new(password_hash_str).map_err(|e| UserError::HashError(e))?;
        let is_correct = Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        Ok(is_correct)
    }
}

#[derive(Debug)]
pub enum UserError
{
    MissingPassword,
    HashError(argon2::password_hash::Error),
}
