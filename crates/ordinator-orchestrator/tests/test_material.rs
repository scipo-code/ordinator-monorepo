mod fixtures;
use chrono::TimeZone;
use chrono::Utc;
use fixtures::work_orders::material_test_work_orders;
use fixtures::workers::three_workers_builder;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::WorkOrderNumber;
use ordinator_orchestrator::logging::setup_logging;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_material_1() -> anyhow::Result<()>
{
    let scheduling_environment = ordinator_test_support::load_scheduling_environment(
        material_test_work_orders,
        three_workers_builder,
    );

    let environment = ordinator_orchestrator::Environment::Test(
        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
    );

    let (orchestrator, _error_receiver, _system_clock_handle) =
        Orchestrator::<TotalSystemSolution>::builder()
            .logging(setup_logging()?)
            .system_clock(&environment)
            .system_configurations_manual(50, 50, 50, 50)
            .scheduling_environment_manual(scheduling_environment)
            .build::<TotalSystemSolution>()?;

    orchestrator.asset_factory(&Asset::Test)?;

    std::thread::sleep(std::time::Duration::from_secs(4));

    let time_environment = orchestrator
        .scheduling_environment
        .lock()
        .unwrap()
        .time_environment
        .periods
        .clone();

    let every_work_order = &orchestrator
        .system_solutions
        .lock()
        .unwrap()
        .get(&Asset::Test)
        .unwrap()
        .load()
        .strategic
        .as_ref()
        .unwrap()
        .every_work_order()
        .clone();

    match every_work_order.get(&WorkOrderNumber(1111990000)).unwrap() {
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly(period) => {
            assert!(period <= &time_environment[0])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project(period) => {
            assert!(period <= &time_environment[0])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => unreachable!(),
    }
    match every_work_order.get(&WorkOrderNumber(1111990001)).unwrap() {
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly(period) => {
            assert!(period <= &time_environment[2])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project(period) => {
            assert!(period <= &time_environment[2])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => unreachable!(),
    }
    match every_work_order.get(&WorkOrderNumber(1111990002)).unwrap() {
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly(period) => {
            assert!(period <= &time_environment[3])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project(period) => {
            assert!(period <= &time_environment[3])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => unreachable!(),
    }
    match every_work_order.get(&WorkOrderNumber(1111990003)).unwrap() {
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly(period) => {
            assert!(period <= &time_environment[3])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project(period) => {
            assert!(period <= &time_environment[3])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => unreachable!(),
    }
    match every_work_order.get(&WorkOrderNumber(1111990004)).unwrap() {
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly(period) => {
            assert!(period <= &time_environment[0])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project(period) => {
            assert!(period <= &time_environment[0])
        }
        ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled => unreachable!(),
    }

    Ok(())
}
