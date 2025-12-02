pub mod auth;
pub mod config;
mod handlers;
mod routes;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use axum::routing::get;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Environment;
use ordinator_orchestrator::Orchestrator;
use tower_http::services::ServeDir;
use tracing::info;
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::Info;
use utoipa::openapi::OpenApiBuilder;
use utoipa::openapi::security::Http;
use utoipa::openapi::security::HttpAuthScheme;
use utoipa::openapi::security::SecurityScheme;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::Config;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::db::AuthDb;
use crate::config::AppConfig;
use crate::routes::api::v1::api_scope;

#[derive(Clone)]
pub struct AppState
{
    pub db: Arc<dyn AuthDb>,
    pub orchestrator: Arc<Orchestrator<TotalSystemSolution>>,
    pub config: &'static AppConfig,
}

pub async fn start_application(
    orchestrator: Arc<Orchestrator<TotalSystemSolution>>,
    db: Arc<dyn AuthDb>,
    config: &'static AppConfig,
    environment: &Environment,
) -> anyhow::Result<()>
{
    let frontend_files = ServeDir::new("./dist/static_files/");

    let app_state = AppState {
        orchestrator,
        db,
        config,
    };

    let app = OpenApiRouter::new()
        // .route_service("/", index_service)
        .nest("/api/v1", api_scope(app_state.clone()).await)
        .nest_service("/app", frontend_files)
        .route("/hello", get(|| async { "Hello, world!" }))
        .with_state(app_state)
        .split_for_parts();

    // Here you can modify the the 'OpenApi' specification
    let swagger_ui_name = match environment {
        Environment::Prod => "Ordinator API Specification (Production Environment)",
        Environment::Test(_) => "Ordinator API Specification (Test Environment)",
    };
    // let openapi = OpenApiBuilder::from(app.1)
    //     .info(Info::new(swagger_ui_name, "0.2.2"))
    //     .build();
    let base_openapi = OpenApiBuilder::from(app.1)
        .info(Info::new(swagger_ui_name, "0.2.2"))
        .build();

    let components = base_openapi.clone().components.unwrap_or_default();
    let components = ComponentsBuilder::from(components)
        .security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
        .build();

    let openapi = OpenApiBuilder::from(base_openapi)
        .components(Some(components))
        .build();

    // Here you can modify the [`SwaggerUi`]
    let swagger_config = Config::new(["/api-doc/openapi.json"])
        .display_request_duration(true)
        .with_syntax_highlight(false)
        .try_it_out_enabled(true);

    let merged_app = app.0.merge(
        SwaggerUi::new("/swagger")
            .config(swagger_config)
            .url("/api-doc/openapi.json", openapi),
    );

    let port = dotenvy::var("SERVER_PORT")?.parse::<u16>()?;
    let address = Ipv4Addr::from_str(dotenvy::var("SERVER_ADDRESS")?.as_str())?;

    let addr = SocketAddr::from((address, port));
    info!(target: "stdout", "System initialized (4 of 4): ordinator-api-server");
    info!(target: "stdout", "Access the API documentation at: http://{}:{}/swagger", &addr.ip(), &addr.port());

    info!(target: "research", "READY");

    axum_server::bind(addr)
        .serve(merged_app.into_make_service())
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
