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
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::TacticalInterface;
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
use ordinator_scheduling_environment::worker_environment::TimeInput;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;

#[derive(Clone, Debug)]
struct TestSystemSolution<Zs: SupervisorInterface + Clone>
{
    supervisor: Zs,
    operational: HashMap<Id, OperationalSolution>,
}

// What should you do here? I believe that I should refactor the
// trait. You already knew that this would be a problem and you
// hated it the first time that you made it. Now it is time to
// do this again. I believe that this will have to do in this
// case.
//
//
// Okay, simply give each of these a dummy.
// QUESTION [ ]
// Should you refactor this now? I think that is a good idea!
//
//
#[derive(PartialEq, PartialOrd, Ord, Clone, Copy, Debug, Eq)]
struct TestStrategic;
impl StrategicInterface for TestStrategic
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
struct TestTactical;

impl TacticalInterface for TestTactical
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&chrono::DateTime<Utc>, &chrono::DateTime<Utc>)>
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
        Resources,
        Vec<ordinator_scheduling_environment::work_order::operation::Work>,
    >
    {
        todo!()
    }
}
impl SystemSolutions for TestSystemSolution<SupervisorSolution>
{
    type Operational = OperationalSolution;
    type Strategic = TestStrategic;
    type Supervisor = SupervisorSolution;
    type Tactical = TestTactical;

    fn new() -> Self
    {
        todo!()
    }

    fn strategic(&self) -> anyhow::Result<&Self::Strategic>
    {
        todo!()
    }

    fn strategic_swap(&mut self, id: &Id, solution: Self::Strategic)
    where
        Self::Strategic: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn tactical_actor_solution(&self) -> anyhow::Result<&Self::Tactical>
    {
        todo!()
    }

    fn tactical_swap(&mut self, id: &Id, solution: Self::Tactical)
    where
        Self::Tactical: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn supervisor_actor_solutions(&self) -> anyhow::Result<&Self::Supervisor>
    {
        Ok(&self.supervisor)
    }

    fn supervisor_swap(&mut self, id: &Id, solution: Self::Supervisor)
    where
        Self::Supervisor: ordinator_orchestrator_actor_traits::Solution,
    {
        todo!()
    }

    fn operational_actor_solutions(&self, id: &Id) -> anyhow::Result<&Self::Operational>
    {
        todo!()
    }

    fn all_operational(&self) -> std::collections::HashSet<Id>
    {
        todo!()
    }

    fn operational_swap(&mut self, id: &Id, solution: Self::Operational)
    where
        Self::Operational: Solution,
    {
        self.operational.insert(id.clone(), solution);
    }
}

impl Solution for TestTactical
{
    type ObjectiveValue = ();
    type Parameters = ();

    fn new(parameters: &Self::Parameters) -> anyhow::Result<Self>
    {
        todo!()
    }

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue)
    {
        todo!()
    }
}
impl Solution for TestStrategic
{
    type ObjectiveValue = ();
    type Parameters = ();

    fn new(parameters: &Self::Parameters) -> anyhow::Result<Self>
    {
        todo!()
    }

    fn update_objective_value(&mut self, other_objective: Self::ObjectiveValue)
    {
        todo!()
    }
}

#[test]
#[ignore]
fn start_operational_actor()
{
    // How do we make a `WorkerEnvironment`. Do we want to rely on files? No! We do
    // not want to rely on raw database JSON/BSON files directly.
    // QUESTION [ ] Should you even be using the `SchedulingEnvironment`? No I do
    // not think that is the best approach here. What other thing could you do?
    // I think that making the a builder for the Actor is the best approach. But
    // how should that look like?
    //
    // TODO [ ] build an OperationalActor and a SupervisorSolution.
    let asset = Asset::Test;
    let asset_string = asset.to_string().to_lowercase();

    let path = format!(
        "temp_scheduling_environment_database/actor_specifications/actor_specification_{asset_string}.toml",
    );
    let path_to_data = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    println!("{}\n{}", path_to_data.display(), line!());
    // println!("{:?}", std::fs::canonicalize(path_to_data.clone()).unwrap());

    // We do not want to test against data files. I think that the best approach
    // here will be to test against something else.
    let worker_environment = ActorEnvironment::builder()
        .actor_environment(Asset::Test, path_to_data)
        // What should be done here? I think that the best approach is to make
        // the system work.
        .unwrap()
        .build();
    // Should you build the actors yourself. Or do something different? I think that
    // the best approach here is to do the same thing again.

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
                    wob.main_work_center(Resources::MtnMech)
                        .operations_builder(10, Resources::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(10.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
                                    )
                                    .earliest_finish_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 2, 7, 0, 0).unwrap(),
                                    )
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    operation_description: Some("TEST".to_string()),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("DF/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // It is clear that you need a thorough understanding of the whole
                                // maintenance process to be able to develop this system.
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
                                .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect("This date is required for constructing a WorkOrderDates object"))
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true))
                        })
                })
                .work_order_builder(WorkOrderNumber(1002), |wob| {
                    wob.main_work_center(Resources::MtnMech)
                        .operations_builder(10, Resources::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(5.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
                                    )
                                    .earliest_finish_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 2, 7, 0, 0).unwrap(),
                                    )
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    operation_description: Some("TEST".to_string()),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("DF/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // It is clear that you need a thorough understanding of the whole
                                // maintenance process to be able to develop this system.
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
                                .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect("This date is required for constructing a WorkOrderDates object"))
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true))
                        })
                })
        })
    .time_environment(time_environment)
        .build();

    // Get it to work first, then change the API
    // TODO [ ] 2025-07-09 make a module that contains SchedulingEnvironment

    let operational_id = &scheduling_environment
        .lock()
        .unwrap()
        .worker_environment
        .actor_specification
        .get(&Asset::Test)
        .unwrap()
        .operational
        .first()
        .unwrap()
        .id
        .clone();
    let supervisor_id = &scheduling_environment
        .lock()
        .unwrap()
        .worker_environment
        .actor_specification
        .get(&Asset::Test)
        .unwrap()
        .supervisors
        .first()
        .unwrap()
        .id
        .clone();

    let operational_state_machine = HashMap::from([(
        (operational_id.clone(), (WorkOrderNumber(1001), 10)),
        Delegate::Assess,
    )]);

    let supervisor = SupervisorSolution::new_from_parts(operational_state_machine);
    dbg!(&supervisor);
    // TODO [ ] 2025-07-08 Make a `builder` for the `SystemSolution`.
    // You need to construct a builder for the `SystemSolution` as well.
    let _system_solution = Arc::new(ArcSwap::new(Arc::new(TestSystemSolution {
        supervisor,
        operational: HashMap::new(),
    })));
    let (sender, receiver) = flume::unbounded();
    let system_configuration = SystemConfigurations::read_all_configs().unwrap();

    let state_link_bus = bus::Bus::new(2).add_rx();
    let communication = Actor::<
        OperationalRequestMessage,
        OperationalResponseMessage,
        OperationalAlgorithm<TestSystemSolution<SupervisorSolution>>,
    >::builder()
    .agent_id(operational_id.clone())
    .scheduling_environment(Arc::clone(&scheduling_environment))
    .algorithm(|ab| {
        ab.id(operational_id.clone())
            // So this function returns a `Result`
            .parameters_and_solution(&scheduling_environment.lock().unwrap())
            .unwrap()
            .system_solution_arc_swap(_system_solution.clone())
    })
    .unwrap()
    .communication(sender, state_link_bus)
    .configurations(system_configuration)
    .build();
    // Okay now this has to work as expected. What is the best path forward
    // here?
    //
    // You have made this now. I think that you
    //
    //
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
    // I am so frustrated about this! I am not really sure what it is that I am
    // doing!
}
