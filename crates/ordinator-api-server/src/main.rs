// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;
mod handlers;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use axum::routing::get;
use chrono::TimeZone;
use chrono_tz::Europe::Copenhagen;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
// use std::fs::File;
// use std::io::Read;
use ordinator_orchestrator::Orchestrator;
use routes::api::v1::api_scope;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use utoipa_axum::router::OpenApiRouter;
#[tokio::main]
async fn main() -> Result<()>
{
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    // TODO [ ] 2025-06-29 turn this into `match
    // dotenvy::var("DEPLOY_ENVIRONMENT");`

    // I think that we should supply the
    // One thing is for sure, the whole thing should be inside of the
    // `Orchestrator::new()` Should the
    let denmark_time = Copenhagen.with_ymd_and_hms(2025, 1, 13, 7, 00, 00).unwrap();
    let current_time = denmark_time.to_utc();

    // ISSUE #000 Turn the nested `std::sync::Mutex` into `tokio::sync::Mutex`
    // ISSUE #000 TODO [ ] 2025-06-29 turn this into `match
    // dotenvy::var("DEPLOY_ENVIRONMENT");`
    let (orchestrator, error_handle, system_clock_handle): (
        Arc<Orchestrator<TotalSystemSolution>>,
        JoinHandle<Result<()>>,
        JoinHandle<()>,
    ) = Orchestrator::new(Some(current_time)).context("Orchestrator could not be created")?;

    // WARN: Manually add `Asset`s here. Everything added here should be done from
    // the API in actual production. So this is only a temporary solution.

    orchestrator.asset_factory(&Asset::DF)?;

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

    let merged_app = app
        .0
        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger").url("/api-doc/openapi.json", app.1));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum_server::bind(addr).serve(merged_app.into_make_service());

    tokio::select! {
        res = server => res?,
        res = error_handle => res??,
        res = system_clock_handle => res?,
    }

    Ok(())
}
