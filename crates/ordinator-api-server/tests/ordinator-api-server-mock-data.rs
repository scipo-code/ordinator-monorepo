mod fixtures;

use chrono::TimeZone;
use chrono::Utc;
use ordinator_api_server::start_application;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::logging::setup_logging;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_complete_system() -> anyhow::Result<()>
{
    let scheduling_environment =
        fixtures::scheduling_environment_master_test::load_scheduling_environment();

    let environment = ordinator_orchestrator::Environment::Test(
        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
    );
    let (orchestrator, error_handle, system_clock_handle) =
        Orchestrator::<TotalSystemSolution>::builder()
            .logging(setup_logging())
            .system_clock(&environment)
            .system_configurations()
            .scheduling_environment_manual(scheduling_environment)
            .build::<TotalSystemSolution>()?;

    orchestrator.asset_factory(&Asset::Test)?;

    dbg!(
        &orchestrator
            .scheduling_environment
            .lock()
            .unwrap()
            .work_orders
            .inner
    );

    let server = start_application(orchestrator, &environment);

    tokio::select! {
        res = server => res.await?,
        res = error_handle => res??,
        res = system_clock_handle => res?,
        // _ = signal::ctrl_c() => {
        //     info!(target: "stdout", "System shutting down");
        // }
    }

    Ok(())
}
