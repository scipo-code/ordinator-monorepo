use axum::Json;
use axum::extract::State;
use ordinator_contracts::auth::AuthConfig;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::auth::provider::Provider;

pub async fn authentication_nest(state: AppState) -> OpenApiRouter<AppState>
{
    let router = OpenApiRouter::new();

    match state.config.auth_provider {
        Provider::Local => router
            .routes(routes!(crate::auth::provider::local::routes::login))
            .routes(routes!(crate::auth::provider::local::routes::refresh)),

        Provider::Azure => todo!(),
    }
    .routes(routes!(crate::auth::routes::auth_config))
    .with_state(state)
}

#[utoipa::path(
    get,
    tag = "Authentication",
    path = "/config",
    responses(
        (status = 200),
    )
)]
pub async fn auth_config(State(state): State<AppState>) -> Json<AuthConfig>
{
    // At some point if we do SaaS we would need to have multitenant support.
    // This would need to be refactored to check the user email and create the
    // appropriate response
    Json(AuthConfig {
        provider: state.config.auth_provider.to_string(),
    })
}
