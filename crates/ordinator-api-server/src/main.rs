use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::TimeZone;
use chrono_tz::Europe::Copenhagen;
use ordinator_api_server::start_application;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::logging::setup_logging;
use tracing::info;

pub const RESEARCH: &str = "research";

#[tokio::main]
async fn main() -> Result<()>
{
    info!(target: "stdout", "System initialized (0 of 4): loading environment");
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    // TODO: Refactor to use `match dotenvy::var("DEPLOY_ENVIRONMENT")` pattern
    // instead of hardcoded value

    let denmark_time = Copenhagen.with_ymd_and_hms(2025, 1, 13, 7, 00, 00).unwrap();
    let current_time = denmark_time.to_utc();

    // ISSUE #000: Replace nested `std::sync::Mutex` with `tokio::sync::Mutex`

    let asset = Asset::DF;
    let environment = ordinator_orchestrator::Environment::Test(current_time);
    let (orchestrator, error_receiver, _system_clock_handle) =
        Orchestrator::<TotalSystemSolution>::builder()
            .logging(setup_logging()?)
            .system_clock(&environment)
            .system_configurations()
            .scheduling_environment_from_database(&asset)?
            .build()?;

    // WARN: Assets are manually added here. In production, assets should be added
    // via the API. This is a temporary solution.

    orchestrator.asset_factory(&asset)?;

    tokio::select! {
        result = start_application(orchestrator.clone(), &environment) => {
            info!(target: "stdout", server_shutdown_message = ?result, "Main server shutting down");
            Ok(())
        }
        result = error_receiver.recv_async() => {
            tracing::error!(ordinator_error_message = ?result, "Ordinator Scheduling Systems experienced a catastrophic error");
            bail!("Ordinator Scheduling Systems experienced a catastrophic error:\n{:?}", result);
        }
    }
}
