pub mod models;
pub mod routes;

use std::sync::Arc;

use async_trait::async_trait;
use ordinator_contracts::auth::LocalAuthPayload;

use crate::auth::current_timestamp;
use crate::auth::db::AuthDb;
use crate::auth::db::DbError;
use crate::auth::models::AuthError;
use crate::auth::models::KEYS;
use crate::auth::models::TokenClaims;
use crate::auth::models::UserError;
use crate::auth::provider::CredentialAuthProvider;
use crate::auth::provider::TokenValidator;

pub struct LocalAuthProvider
{
    db: Arc<dyn AuthDb>,
}

impl LocalAuthProvider
{
    pub fn new(db: Arc<dyn AuthDb>) -> Self
    {
        Self { db }
    }
}

#[async_trait]
impl CredentialAuthProvider for LocalAuthProvider
{
    type Credentials = LocalAuthPayload;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<crate::auth::models::TokenClaims, crate::auth::models::AuthError>
    {
        let user = self
            .db
            .get_user_by_email(&credentials.client_id)
            .await
            .map_err(|e| match e {
                DbError::NotFound => {
                    tracing::warn!(target: "auth", error=?e, email=%credentials.client_id, "Login attempt. User Not found error.");
                    AuthError::UserNotFound /* Do not leak user does not */
                }
                // exist
                _ => {
                    tracing::error!(target: "auth", error=?e, email=%credentials.client_id, "Database error during user lookup.");
                    AuthError::InternalError
                },
            })?
            .ok_or_else(|| {
                tracing::warn!(target: "auth", email=%credentials.client_id, "Login attempt. Wrong credentials: User email.");
                AuthError::WrongCredentials
            })?;

        let is_valid = user.verify_password(&credentials.client_secret).map_err(|e| match e{
            UserError::MissingPassword => {
                tracing::error!(target: "auth", error=?e, user_email = %user.email, "Local user missing password hash - data integrity error");

                AuthError::InternalError}, // User exists without password = config error. This should not happen for local provider
            UserError::HashError(e) => {
                tracing::error!(target: "auth", error=?e, user_email = %user.email, "Password verification failed");
                AuthError::InternalError
            }})?;

        if !is_valid {
            tracing::warn!(target: "auth", user_email = %user.email, "Invalid login credentials: Password");
            return Err(AuthError::WrongCredentials);
        }

        if !user.is_active {
            tracing::warn!(target: "auth", user_email = %user.email, "User inactive credentials login attempt");
            return Err(AuthError::UserInactive);
        }

        let claims = TokenClaims::new(
            user.email.clone(),
            user.role.clone(),
            user.assets.clone(),
            user.provider.clone(),
        );

        tracing::info!(target: "auth", user_email=&user.email, role=?user.role, "User authenticated succesfully");
        Ok(claims)
    }
}

#[async_trait]
impl TokenValidator for LocalAuthProvider
{
    async fn validate_token(
        &self,
        token: &str,
    ) -> Result<crate::auth::models::TokenClaims, crate::auth::models::AuthError>
    {
        use jsonwebtoken::Validation;
        use jsonwebtoken::decode;

        let token_data = decode::<TokenClaims>(token, &KEYS.decoding, &Validation::default())
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })?;

        if token_data.claims.exp < current_timestamp() {
            return Err(AuthError::TokenExpired);
        }

        Ok(token_data.claims)
    }
}
