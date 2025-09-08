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
use utoipa::openapi::Info;
use utoipa::openapi::OpenApiBuilder;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::Config;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::api::v1::api_scope;

pub async fn start_application(
    orchestrator: Arc<Orchestrator<TotalSystemSolution>>,
    environment: &Environment,
) -> anyhow::Result<()>
{
    // let index_service =
    // get_service(ServeDir::new("./static_files/index")).handle_error(     |err:
    // std::io::Error| async move {         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("Unhandled internal server error: {err}"),
    //         )
    //     },
    // );

    let scheduler_files = ServeDir::new("./dist/static_files/scheduler/");
    let supervisor_files = ServeDir::new("./dist/static_files/supervisor//");

    let app = OpenApiRouter::new()
        // .route_service("/", index_service)
        .nest("/api/v1", api_scope(orchestrator.clone()).await)
        .nest_service("/scheduler", scheduler_files)
        .nest_service("/supervisor", supervisor_files)
        .route("/hello", get(|| async { "Hello, world!" }))
        .with_state(orchestrator)
        .split_for_parts();

    // Here you can modify the the 'OpenApi' specification
    let swagger_ui_name = match environment {
        Environment::Prod => "Ordinator API Specification (Production Environment)",
        Environment::Test(_) => "Ordinator API Specification (Test Environment)",
    };
    let openapi = OpenApiBuilder::from(app.1)
        .info(Info::new(swagger_ui_name, "0.2.2"))
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
    axum_server::bind(addr)
        .serve(merged_app.into_make_service())
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
