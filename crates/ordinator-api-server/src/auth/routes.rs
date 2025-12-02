use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::auth::provider::Provider;

#[derive(Serialize)]
pub struct LoginResponse
{
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

pub async fn authentication_nest(state: AppState) -> OpenApiRouter<AppState>
{
    let router = OpenApiRouter::new();

    match state.config.auth_provider {
        Provider::Local => router
            .routes(routes!(crate::auth::provider::local::routes::login))
            .routes(routes!(crate::auth::provider::local::routes::refresh)),

        Provider::Azure => todo!(),
    }
    .with_state(state)
}
