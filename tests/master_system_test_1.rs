mod fixtures;

use anyhow::anyhow;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_api_server::start_application;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::logging::setup_logging;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
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

    // The issue is that you should not handle the errors in here. But handle them
    // elsewhere.
    tokio::spawn(async move {
        if let Err(e) = system_clock_handle.await {
            eprintln!("Clock error: {}", e);
        }
    });
    // Handle this later!
    // FIX START HERE TOMORROW.
    tokio::spawn()

    orchestrator
        .error_sender
        .send(anyhow!("ERROR IN MAIN"))
        .unwrap();

    start_application(orchestrator.clone(), &environment).await
}
