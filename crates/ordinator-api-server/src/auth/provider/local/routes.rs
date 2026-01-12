use axum::Json;
use axum::extract::State;
use jsonwebtoken::Header;
use jsonwebtoken::encode;
use ordinator_contracts::AssetNames;
use ordinator_contracts::auth::LocalAuthPayload;
use ordinator_contracts::auth::LoginResponse;

use crate::AppState;
use crate::auth::models::AuthError;
use crate::auth::models::KEYS;
use crate::auth::models::RefreshTokenClaims;
use crate::auth::models::TokenClaims;
use crate::auth::provider::CredentialAuthProvider;
use crate::auth::provider::local::LocalAuthProvider;
use crate::config::get_config;

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/login",
    request_body(content = LocalAuthPayload, description = "JSON with the clientid and optionally secret", content_type = "application/json", example = json!({"client_id": "example@ordinator.com", "client_secret": "secret_password"})),
    responses(
        (status = 200),
        (status = 400, body = AuthError),
        (status = 401, body = AuthError),
        (status = 403, body = AuthError),
        (status = 404, body = AuthError),
        (status = 500, body = AuthError),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<LocalAuthPayload>,
) -> Result<Json<LoginResponse>, AuthError>
{
    let config = get_config();
    let provider = LocalAuthProvider::new(state.db.clone());
    let claims = provider.authenticate(credentials).await?;

    let access_token = encode(&Header::new(config.jwt_algorithm), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let refresh_claims = RefreshTokenClaims::new(claims.sub.clone(), claims.provider.clone());

    let refresh_token = encode(
        &Header::new(config.jwt_algorithm),
        &refresh_claims,
        &KEYS.encoding,
    )
    .map_err(|_| AuthError::TokenCreation)?;

    let assets = claims
        .assets
        .iter()
        .map(|a| AssetNames::from(a.clone()))
        .collect::<Vec<AssetNames>>();

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        role: claims.role.to_string(),
        assets,
    }))
}

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/refresh",
    request_body(content = RefreshTokenClaims, description = "JSON with the clientid and optionally secret", content_type = "application/json"),
    responses(
        (status = 200),
        (status = 400, body = AuthError),
        (status = 401, body = AuthError),
        (status = 403, body = AuthError),
        (status = 404, body = AuthError),
        (status = 500, body = AuthError),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    refresh_claims: RefreshTokenClaims,
) -> Result<Json<LoginResponse>, AuthError>
{
    if refresh_claims.token_type != "refresh" {
        return Err(AuthError::InvalidToken);
    }

    let user = state
        .db
        .get_user_by_email(&refresh_claims.sub)
        .await
        .map_err(|_| AuthError::InternalError)?
        .ok_or(AuthError::InvalidToken)?;

    if !user.is_active {
        return Err(AuthError::UserInactive);
    }

    let claims = TokenClaims::new(user.email, user.role, user.assets, user.provider);

    let access_token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let new_refresh_claims = RefreshTokenClaims::new(claims.sub.clone(), claims.provider.clone());

    let refresh_token = encode(&Header::default(), &new_refresh_claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let assets = claims
        .assets
        .iter()
        .map(|a| AssetNames::from(a.clone()))
        .collect::<Vec<AssetNames>>();

    Ok(Json(LoginResponse {
        access_token: access_token,
        refresh_token: refresh_token,
        token_type: "Bearer".to_string(),
        role: claims.role.to_string(),
        assets,
    }))
}
