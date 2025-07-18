mod handlers;
mod routes;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Environment;
use ordinator_orchestrator::Orchestrator;
use tower_http::services::ServeDir;
use tracing::info;
use utoipa::openapi::Info;
use utoipa::openapi::OpenApiBuilder;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::Config;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::api::v1::api_scope;

pub async fn start_application(
    orchestrator: Arc<Orchestrator<TotalSystemSolution>>,
    environment: &Environment,
) -> impl Future<Output = std::result::Result<(), std::io::Error>>
{
    let scheduler_files = ServeDir::new("./static_files/scheduler/dist");
    let supervisor_files =
        ServeDir::new("./static_files/supervisor/dist/supervisor-calendar/browser");

    let app = OpenApiRouter::new()
        .nest("/api/v1", api_scope(orchestrator.clone()).await)
        .nest_service("/scheduler", scheduler_files)
        .nest_service("/supervisor", supervisor_files)
        .route("/hello", get(|| async { "Hello, world!" }))
        .with_state(orchestrator)
        .split_for_parts();

    // Here you can modify the the 'OpenApi' specification
    let swagger_ui_name = match environment {
        Environment::Prod => "Ordinator API Specification (Production Environment)",
        Environment::Test(_) => "Ordinator API Specifivation (Test Environment)",
    };
    let openapi = OpenApiBuilder::from(app.1)
        .info(Info::new(swagger_ui_name, "0.2.2"))
        .build();

    // Here you can modify the [`SwaggerUi`]
    let swagger_config = Config::new(["/api-doc/openapi.json"])
        .display_request_duration(true)
        .try_it_out_enabled(true);

    let merged_app = app.0.merge(
        SwaggerUi::new("/swagger")
            .config(swagger_config)
            .url("/api-doc/openapi.json", openapi),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum_server::bind(addr).serve(merged_app.into_make_service());

    info!(target: "stdout", "System initialized (4 of 4): ordinator-api-server");
    server
}
