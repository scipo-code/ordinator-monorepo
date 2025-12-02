// use axum::Json;
// use jsonwebtoken::Header;
// use jsonwebtoken::encode;

// use crate::auth::models::AuthBody;
// use crate::auth::models::AuthError;
// use crate::auth::models::AuthPayload;
// use crate::auth::models::TokenDetails;
// use crate::auth::models::KEYS;

use std::sync::LazyLock;

pub static JWT_CONFIG: LazyLock<JwtConfig> = LazyLock::new(|| JwtConfig::from_env());

pub struct JwtConfig
{
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_expiration: i64, // seconds
    pub refresh_token_expiration: i64,
}

impl JwtConfig
{
    pub fn from_env() -> Self
    {
        Self {
            secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            issuer: std::env::var("JWT_ISSUER").unwrap_or_else(|_| "ordinator-api".to_string()),
            audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "ordinator-client".to_string()),
            access_token_expiration: std::env::var("JWT_ACCESS_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            refresh_token_expiration: std::env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .unwrap_or(604800),
        }
    }
}
// pub async fn authorize(Json(payload): Json<AuthPayload>) ->
// Result<Json<AuthBody>, AuthError> {
//     if payload.client_id.is_empty() || payload.client_secret.is_empty() {
//         return Err(AuthError::MissingCredentials);
//     }

//     // Here we would connect to a database to authenticate
//     let claims = match (payload.client_id.as_str(),
// payload.client_secret.as_str()) {         ("scheduler", "pass") =>
// Claims::new(             "sched@ordinator.com".to_owned(),
//             crate::auth::models::UserRole::Scheduler,
//             vec!["test".to_string(), "df".to_string()],
//         ),
//         ("supervisor", "pass") => Claims::new(
//             "super@ordinator.com".to_owned(),
//             crate::auth::models::UserRole::Supervisor,
//             vec!["test".to_string(), "df".to_string()],
//         ),
//         ("tech", "pass") => Claims::new(
//             "tech@ordinator.com".to_owned(),
//             crate::auth::models::UserRole::Technician,
//             vec!["test".to_string(), "df".to_string()],
//         ),
//         _ => return Err(AuthError::WrongCredentials),
//     };

//     let token = encode(&Header::default(), &claims, &KEYS.encoding)
//         .map_err(|_| AuthError::TokenCreation)?;

//     Ok(Json(AuthBody::new(token)))
// }
