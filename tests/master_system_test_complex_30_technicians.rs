mod fixtures;

use anyhow::bail;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_api_server::start_application;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::logging::setup_logging;
use tracing::info;

use crate::fixtures::work_orders::phd_work_orders_complex_11_resources::phd_work_order_builder_complex_11_resources;
use crate::fixtures::workers::phd_technicians_30::phd_workers_builder;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn master_system_test_complex_30_technicians() -> anyhow::Result<()>
{
    let scheduling_environment = ordinator_test_support::load_scheduling_environment(
        phd_work_order_builder_complex_11_resources,
        phd_workers_builder,
    );

    let environment = ordinator_orchestrator::Environment::Test(
        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
    );

    let (orchestrator, error_receiver, _system_clock_handle) =
        Orchestrator::<TotalSystemSolution>::builder()
            .logging(setup_logging()?)
            .system_clock(&environment)
            .system_configurations()
            .scheduling_environment_manual(scheduling_environment)
            .build::<TotalSystemSolution>()?;

    orchestrator.asset_factory(&Asset::Test)?;

    tokio::select! {
        result = start_application(orchestrator.clone(), &environment) => {
            info!(target: "stdout", server_shutdown_message = ?result, "Main server shutting down");
            Ok(())
        }
        result = error_receiver.recv_async() => {
            tracing::error!(ordinator_error_message = ?result, "Ordinator
            Scheduling Systems experienced a catastraphoic error");
            bail!("Ordinator Scheduling Systems experienced a catastraphoic
            error:\n{:?}", result );
        }
    }
}
