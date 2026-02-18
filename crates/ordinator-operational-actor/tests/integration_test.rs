#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::option::Option;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::NaiveDate;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_operational_actor::algorithm::OperationalAlgorithm;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
use ordinator_operational_actor::messages::OperationalRequestMessage;
use ordinator_operational_actor::messages::OperationalResponseMessage;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::WeeklyInterface;
use ordinator_orchestrator_actor_traits::DailyInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::ProjectInterface;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::create_time_environment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoDetail;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::revision::Revision;
use ordinator_scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::ActorEnvironment;
use ordinator_scheduling_environment::worker_environment::ActorSpecification;
use ordinator_scheduling_environment::worker_environment::ActorSpecifications;
use ordinator_scheduling_environment::worker_environment::TimeInput;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_supervisor_actor::algorithm::supervisor_solution::DailySolution;

#[derive(Clone, Debug)]
struct TestSystemSolution<Zs: DailyInterface + Clone>
{
    supervisor: Option<Zs>,
    operational: HashMap<ActorCompositeId, SolutionState<OperationalSolution>>,
}

// TODO: Refactor trait implementation - currently using dummy implementations
#[derive(PartialEq, PartialOrd, Ord, Clone, Copy, Debug, Eq)]
struct TestWeekly;
impl WeeklyInterface for TestWeekly
{
    fn scheduled_task(
        &self,
        work_order_number: &WorkOrderNumber,
    ) -> Option<&WhereIsWorkOrder<Period>>
    {
        todo!()
    }

    fn supervisor_tasks(&self, periods: &[Period]) -> HashMap<WorkOrderNumber, Period>
    {
        todo!()
    }

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, Period>
    {
        todo!()
    }
}

#[derive(PartialEq, PartialOrd, Ord, Clone, Copy, Debug, Eq)]
struct TestProject;

impl ProjectInterface for TestProject
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&NaiveDate, &NaiveDate)>
    {
        todo!()
    }

    fn tactical_period<'a>(
        &self,
        work_order_number: &WorkOrderNumber,
        periods: &'a [Period],
    ) -> Option<&'a Period>
    {
        todo!()
    }

    fn all_scheduled_tasks(
        &self,
    ) -> HashMap<
        WorkOrderNumber,
        std::collections::BTreeMap<
            ordinator_scheduling_environment::work_order::operation::ActivityNumber,
            ordinator_scheduling_environment::time_environment::day::Day,
        >,
    >
    {
        todo!()
    }

    fn tactical_loadings(
        &self,
    ) -> std::collections::BTreeMap<
        Skill,
        Vec<ordinator_scheduling_environment::work_order::operation::Work>,
    >
    {
        todo!()
    }
}
impl SystemSolutions for TestSystemSolution<DailySolution>
{
    type Operational = OperationalSolution;
    type Weekly = TestWeekly;
    type Daily = DailySolution;
    type Project = TestProject;

    fn new() -> Self
    {
        todo!()
    }

    fn strategic(&self) -> anyhow::Result<&Self::Weekly>
    {
        todo!()
    }

    fn strategic_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<TestWeekly>)
    where
        Self::Weekly: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn tactical_actor_solution(&self) -> anyhow::Result<&Self::Project>
    {
        todo!()
    }

    fn tactical_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<TestProject>)
    where
        Self::Project: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn supervisor_actor_solutions(&self) -> anyhow::Result<&Self::Daily>
    {
        Ok(self.supervisor.as_ref().unwrap())
    }

    fn supervisor_swap(
        &mut self,
        id: &ActorCompositeId,
        solution: SolutionState<DailySolution>,
    ) where
        Self::Daily: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn operational_actor_solutions(
        &self,
        id: &ActorCompositeId,
    ) -> anyhow::Result<&Self::Operational>
    {
        todo!()
    }

    fn all_operational(&self) -> std::collections::HashSet<ActorCompositeId>
    {
        todo!()
    }

    fn operational_swap(
        &mut self,
        id: &ActorCompositeId,
        solution: SolutionState<OperationalSolution>,
    ) where
        Self::Operational: Solution,
    {
        self.operational.insert(id.clone(), solution);
    }
}

impl Solution for TestProject
{
    type Objective = ();
    type Parameters = ();

    fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
    {
        todo!()
    }

    fn update_objective(&mut self, other_objective: Self::Objective)
    {
        todo!()
    }
}
impl Solution for TestWeekly
{
    type Objective = ();
    type Parameters = ();

    fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
    {
        todo!()
    }

    fn update_objective(&mut self, other_objective: Self::Objective)
    {
        todo!()
    }
}

#[test]
#[ignore]
fn start_operational_actor()
{
    // TODO: Build OperationalActor and DailySolution using builder pattern instead of file-based configuration
    let asset = Asset::Test;
    let asset_string = asset.to_string().to_lowercase();

    let path = format!(
        "temp_scheduling_environment_database/actor_specifications/actor_specification_{asset_string}.toml",
    );
    let path_to_data = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    println!("{}\n{}", path_to_data.display(), line!());
    // println!("{:?}", std::fs::canonicalize(path_to_data.clone()).unwrap());

    let worker_environment = ActorEnvironment::<dyn ActorSpecification>::builder()
        .add_actor_specification(
            asset,
            Box::new(ActorSpecifications::actor_specification_from_toml(path_to_data).unwrap()),
        )
        .build();

    let time_input = TimeInput {
        number_of_periods: 3,
        number_of_days: 42,
    };
    let time_environment = create_time_environment(
        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
        &time_input,
    );

    let scheduling_environment = SchedulingEnvironment::builder()
        .worker_environment(worker_environment)
        .work_orders_builder(|wo_builder| {
            wo_builder
                .work_order_builder(WorkOrderNumber(1001), |wob| {
                    wob.main_work_center(Skill::MtnMech)
                        .operations_builder(10, Skill::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(10.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                    .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .unwrap()
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("DF/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // Developing this system requires thorough understanding of the maintenance process
                                .work_order_info_detail(WorkOrderInfoDetail {
                                    subnetwork: "123".to_string(),
                                    maintenance_plan: "PLAN TEST".to_string(),
                                    planner_group: "TEST_GROUP".to_string(),
                                    maintenance_plant: "TEST".to_string(),
                                    pm_collective: "TEST".to_string(),
                                    room: "TEST_ROOM".to_string(),
                                })
                        })
                        .work_order_dates_builder(|wodb| {
                            wodb.duration(TimeDelta::days(1))
                                .basic_start_from_ymd(2025, 1, 1)
                                .basic_finish_from_ymd(2025, 1, 1)
                                .earliest_allowed_start_from_ymd(2025, 1, 1)
                                .latest_allowed_finish_from_ymd(2025, 5, 1)
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true))
                        })
                })
                .work_order_builder(WorkOrderNumber(1002), |wob| {
                    wob.main_work_center(Skill::MtnMech)
                        .operations_builder(10, Skill::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(5.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                    .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .unwrap()
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("DF/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // Developing this system requires thorough understanding of the maintenance process
                                .work_order_info_detail(WorkOrderInfoDetail {
                                    subnetwork: "123".to_string(),
                                    maintenance_plan: "PLAN TEST".to_string(),
                                    planner_group: "TEST_GROUP".to_string(),
                                    maintenance_plant: "TEST".to_string(),
                                    pm_collective: "TEST".to_string(),
                                    room: "TEST_ROOM".to_string(),
                                })
                        })
                        .work_order_dates_builder(|wodb| {
                            wodb.duration(TimeDelta::days(1))
                                .basic_start_from_ymd(2025, 1, 1)
                                .basic_finish_from_ymd(2025, 1, 1)
                                .earliest_allowed_start_from_ymd(2025, 1, 1)
                                .latest_allowed_finish_from_ymd(2025, 5, 1)
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true))
                        })
                })
        })
        .time_environment(time_environment)
        .build()
        .expect("Could not build SchedulingEnvironment");

    // TODO: Create module for SchedulingEnvironment (2025-07-09)

    let operational_id = &scheduling_environment
        .lock()
        .unwrap()
        .worker_environment
        .actor_specification
        .get(&Asset::Test)
        .unwrap()
        .operational()
        .iter()
        .next()
        .unwrap()
        .0
        .clone();

    let supervisor_id = &scheduling_environment
        .lock()
        .unwrap()
        .worker_environment
        .actor_specification
        .get(&Asset::Test)
        .unwrap()
        .supervisor()
        .first()
        .unwrap()
        .id
        .clone();

    let _system_solution = Arc::new(ArcSwap::new(Arc::new(TestSystemSolution {
        supervisor: None,
        operational: HashMap::new(),
    })));

    let operational_id: ActorCompositeId = _system_solution
        .load()
        .operational
        .keys()
        .next()
        .unwrap()
        .clone();
    let operational_state_machine = HashMap::from([(
        (operational_id.clone(), (WorkOrderNumber(1001), 10)),
        Delegate::Assess,
    )]);

    let supervisor = DailySolution::new_from_parts(operational_state_machine);
    dbg!(&supervisor);
    // TODO: Implement builder for SystemSolution (2025-07-08)
    let (sender, receiver) = flume::unbounded();
    let system_configuration = SystemConfigurations::read_all_configs().unwrap();

    let state_link_bus = bus::Bus::new(2).add_rx();
    let communication = Actor::<
        OperationalRequestMessage,
        OperationalResponseMessage,
        OperationalAlgorithm<TestSystemSolution<DailySolution>>,
    >::builder()
    .agent_id(operational_id.clone())
    .scheduling_environment(Arc::clone(&scheduling_environment))
    .algorithm(|ab| {
        ab.id(operational_id.clone())
            .parameters_and_solution(&scheduling_environment.lock().unwrap())
            .unwrap()
            .system_solution_arc_swap(_system_solution.clone())
    })
    .unwrap()
    .communication(sender, state_link_bus)
    .configurations(system_configuration)
    .build();

    match receiver.recv() {
        Ok(t) => panic!(),
        Err(e) => {
            dbg!(e);
            panic!();
        }
    };

    // assert!(
    //     _system_solution
    //         .load()
    //         .operational
    //         .get(&operational_id)
    //         .unwrap()
    //         .scheduled_work_order_activities
    //         .iter()
    //         .any(|d| d.0 == (WorkOrderNumber(1001), 10))
    // );
}
