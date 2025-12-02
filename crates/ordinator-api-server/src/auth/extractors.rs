use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::models::AuthError;
use crate::auth::models::TokenClaims;
use crate::auth::models::UserRole;

pub struct AsScheduler(pub TokenClaims);
pub struct AsSupervisor(pub TokenClaims);
pub struct AsTechnician(pub TokenClaims);
pub struct AsAdmin(pub TokenClaims);

impl<S> FromRequestParts<S> for AsScheduler
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>
    {
        if dotenvy::var("DEV_BYPASS_AUTH").ok().as_deref() == Some("1") {
            return Ok(AsScheduler(TokenClaims::dev_default(UserRole::Scheduler)));
        }
        let claims = TokenClaims::from_request_parts(parts, state).await?;
        match claims.role {
            UserRole::Scheduler => Ok(AsScheduler(claims)),
            _ => Err(AuthError::Forbidden),
        }
    }
}
impl<S> FromRequestParts<S> for AsSupervisor
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>
    {
        if dotenvy::var("DEV_BYPASS_AUTH").ok().as_deref() == Some("1") {
            return Ok(AsSupervisor(TokenClaims::dev_default(UserRole::Supervisor)));
        }
        let claims = TokenClaims::from_request_parts(parts, state).await?;
        match claims.role {
            UserRole::Supervisor => Ok(AsSupervisor(claims)),
            _ => Err(AuthError::Forbidden),
        }
    }
}
impl<S> FromRequestParts<S> for AsTechnician
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>
    {
        if dotenvy::var("DEV_BYPASS_AUTH").ok().as_deref() == Some("1") {
            return Ok(AsTechnician(TokenClaims::dev_default(UserRole::Technician)));
        }
        let claims = TokenClaims::from_request_parts(parts, state).await?;
        match claims.role {
            UserRole::Technician => Ok(AsTechnician(claims)),
            _ => Err(AuthError::Forbidden),
        }
    }
}
impl<S> FromRequestParts<S> for AsAdmin
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>
    {
        if dotenvy::var("DEV_BYPASS_AUTH").ok().as_deref() == Some("1") {
            return Ok(AsAdmin(TokenClaims::dev_default(UserRole::Admin)));
        }
        let claims = TokenClaims::from_request_parts(parts, state).await?;
        match claims.role {
            UserRole::Admin => Ok(AsAdmin(claims)),
            _ => Err(AuthError::Forbidden),
        }
    }
}
