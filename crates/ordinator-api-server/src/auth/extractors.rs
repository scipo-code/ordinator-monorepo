// WARN: This was an attempt to add roles to the Authenticated users. But we
// need to ask who does what before we infer this. For now just use TokenClaims.
// A potential scalable solution is OPA Open Policy Agent (open source
// authorization repo)

// use axum::extract::FromRequestParts;
// use axum::http::request::Parts;

// use crate::auth::models::AuthError;
// use crate::auth::models::TokenClaims;
// use crate::auth::models::UserRole;
// use crate::config::get_config;

// pub struct Authenticated(pub TokenClaims);

// impl<S> FromRequestParts<S> for Authenticated
// where
//     S: Send + Sync,
// {
//     #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A
// rejection is"]     #[doc = " a kind of error that can be converted into a
// response."]     type Rejection = AuthError;

//     #[doc = " Perform the extraction."]
//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self,
// Self::Rejection>     {
//         let config = get_config();

//         if config.dev_bypass_auth {
//             return
// Ok(Authenticated(TokenClaims::dev_default(UserRole::Scheduler)));         }

//         let claims = TokenClaims::from_request_parts(parts, state).await?;
//         Ok(Authenticated(claims))
//     }
// }

// pub struct AuthenticatedAsScheduler(pub TokenClaims);
// pub struct AuthenticatedAsSupervisor(pub TokenClaims);
// pub struct AuthenticatedAsTechnician(pub TokenClaims);
// pub struct AuthenticatedAsAdmin(pub TokenClaims);

// impl<S> FromRequestParts<S> for AuthenticatedAsScheduler
// where
//     S: Send + Sync,
// {
//     type Rejection = AuthError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self,
// Self::Rejection>     {
//         let config = get_config();
//         if config.dev_bypass_auth {
//             return
// Ok(AuthenticatedAsScheduler(TokenClaims::dev_default(UserRole::Scheduler)));
//         }
//         let claims = TokenClaims::from_request_parts(parts, state).await?;
//         match claims.role {
//             UserRole::Scheduler => Ok(AuthenticatedAsScheduler(claims)),
//             _ => Err(AuthError::Forbidden),
//         }
//     }
// }
// impl<S> FromRequestParts<S> for AuthenticatedAsSupervisor
// where
//     S: Send + Sync,
// {
//     type Rejection = AuthError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self,
// Self::Rejection>     {
//         let config = get_config();
//         if config.dev_bypass_auth {
//             return
// Ok(AuthenticatedAsSupervisor(TokenClaims::dev_default(UserRole::Supervisor)));
//         }
//         let claims = TokenClaims::from_request_parts(parts, state).await?;
//         match claims.role {
//             UserRole::Supervisor => Ok(AuthenticatedAsSupervisor(claims)),
//             _ => Err(AuthError::Forbidden),
//         }
//     }
// }
// impl<S> FromRequestParts<S> for AuthenticatedAsTechnician
// where
//     S: Send + Sync,
// {
//     type Rejection = AuthError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self,
// Self::Rejection>     {
//         let config = get_config();
//         if config.dev_bypass_auth {
//             return
// Ok(AuthenticatedAsTechnician(TokenClaims::dev_default(UserRole::Technician)));
//         }
//         let claims = TokenClaims::from_request_parts(parts, state).await?;
//         match claims.role {
//             UserRole::Technician => Ok(AuthenticatedAsTechnician(claims)),
//             _ => Err(AuthError::Forbidden),
//         }
//     }
// }
// impl<S> FromRequestParts<S> for AuthenticatedAsAdmin
// where
//     S: Send + Sync,
// {
//     type Rejection = AuthError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self,
// Self::Rejection>     {
//         let config = get_config();
//         if config.dev_bypass_auth {
//             return
// Ok(AuthenticatedAsAdmin(TokenClaims::dev_default(UserRole::Admin)));
//         }
//         let claims = TokenClaims::from_request_parts(parts, state).await?;
//         match claims.role {
//             UserRole::Admin => Ok(AuthenticatedAsAdmin(claims)),
//             _ => Err(AuthError::Forbidden),
//         }
//     }
// }
