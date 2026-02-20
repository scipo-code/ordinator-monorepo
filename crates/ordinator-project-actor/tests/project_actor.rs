// use std::collections::HashMap;
// use std::str::FromStr;

// use chrono::Days;
// use ordinator_actor_core::algorithm::Algorithm;
// use ordinator_orchestrator_actor_traits::Solution;
// use ordinator_orchestrator_actor_traits::SystemSolution;
// use ordinator_orchestrator_actor_traits::SystemSolutions;
// use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
// use ordinator_scheduling_environment::time_environment::day::Day;
// use ordinator_scheduling_environment::time_environment::period::Period;
// use ordinator_scheduling_environment::work_order::WorkOrderNumber;
// use ordinator_scheduling_environment::work_order::operation::Work;
// use ordinator_scheduling_environment::worker_environment::ProjectOptions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
// use ordinator_scheduling_environment::worker_environment::resources::Resources;
// use ordinator_project_actor::algorithm::project_parameters::OperationParameter;
// use ordinator_project_actor::algorithm::project_parameters::ProjectParameter;
// use ordinator_project_actor::algorithm::project_parameters::ProjectParameters;
// use ordinator_project_actor::algorithm::project_resources::ProjectResources;
// use ordinator_project_actor::algorithm::project_solution::OperationSolution;
// use ordinator_project_actor::algorithm::project_solution::ProjectScheduledOperations;
// use ordinator_project_actor::algorithm::project_solution::ProjectSolution;
// use strum::IntoEnumIterator;

// TODO: Add test with stubs for project actor construction

#[test]
#[ignore]
fn test_calculate_objective_value()
{
    // let work_order_number = WorkOrderNumber(2100000001);
    // let activity_number = 1;
    // let first_period = Period::from_str("2024-W13-14").unwrap();

    // let project_days = |number_of_days: u32| -> Vec<Day> {
    //     let mut days: Vec<Day> = Vec::new();
    //     let mut date = first_period.start_date().to_owned();
    //     for day_index in 0..number_of_days {
    //         days.push(Day::new(day_index as usize, date.to_owned()));
    //         date = date.checked_add_days(Days::new(1)).unwrap();
    //     }
    //     days
    // };
    // Work Order
    // Resources::MtnMech,
    // 10,
    // vec![],
    // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),

    // Operation
    // 1,
    // Work::from(1.0),
    // Work::from(1.0),
    // Work::from(1.0),
    // Resources::MtnMech,
    // SchedulingEnvironment requires arc mutex protection to ensure thread-safe access
    let _scheduling_environment = SchedulingEnvironment::builder()
        .time_environment_builder(|ib| ib.project_days("2025-02-22T07:00:00Z", 56))
        .build();

    // TODO: Add functions to create SchedulingEnvironment

    let _id = ActorCompositeId::default();

    // let system_configurations =
    // SystemConfigurations::read_all_configs().unwrap().load();

    // TODO: Move to integration testing with proper SharedSolution initialization
    // let algorithm: ProjectAlgorithm = Algorithm::builder()
    //     .id(id)
    //     .parameters_and_solution(
    //         &system_configurations,
    //         &scheduling_environment.lock().unwrap(),
    //     )
    //     .unwrap()
    //     .build();

    // TODO: Extract strategic options from SystemConfigurations
    // TODO: Pass system configuration to Orchestrator and Agents
    // TODO: Add SystemConfiguration methods for extracting required configs

    // let mut project_algorithm = Algorithm::new(
    //     &id,
    //     solution,
    //     parameters,
    //     ArcSwapSharedSolution::default().into(),
    // );

    // TODO: Refactor to inject configs into the program rather than hardcoding
    // let operation_parameter = OperationParameter::new(work_order_number,
    // operation);

    // let operation_solution = OperationSolution::new(
    //     vec![(
    //         project_algorithm.project_days[27].clone(),
    //         Work::from(1.0),
    //     )],
    //     Resources::MtnMech,
    //     operation_parameter.number,
    //     operation_parameter.work_remaining,
    //     work_order_number,
    //     activity_number,
    // );

    // let mut operation_parameters = HashMap::new();
    // operation_parameters.insert(activity_number, operation_parameter);

    // let mut operation_solutions = HashMap::new();
    // operation_solutions.insert(1, operation_solution);

    // // We simply have to make
    // let optimized_project_work_order =
    //     ProjectParameter::new(&work_order, operation_parameters);

    // project_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, optimized_project_work_order);

    // project_algorithm.calculate_objective_value().unwrap();

    // // assert_eq!(project_algorithm.objective_value().0, 270);
}

// TODO: Implement test_schedule_1 properly
#[test]
fn test_schedule_1()
{
    // let work_order_number = WorkOrderNumber(2100000001);
    // let first_period = Period::from_str("2024-W13-14").unwrap();

    // let project_days = |number_of_days: u32| -> Vec<Day> {
    //     let mut days: Vec<Day> = Vec::new();
    //     let mut date = first_period.start_date().to_owned();
    //     for day_index in 0..number_of_days {
    //         days.push(Day::new(day_index as usize, date.to_owned()));
    //         date = date.checked_add_days(Days::new(1)).unwrap();
    //     }
    //     days
    // };

    // NOTE: Prioritize system operational stability over test implementation
    // let mut project_algorithm =
    // Algorithm::builder().new(     project_days(56),
    //     ProjectResources::new_from_data(
    //         Resources::iter().collect(),
    //         project_days(56),
    //         Work::from(0.0),
    //     ),
    //     ProjectResources::new_from_data(
    //         Resources::iter().collect(),
    //         project_days(56),
    //         Work::from(0.0),
    //     ),
    //     ArcSwapSharedSolution::default().into(),
    // );
    // // Work Order
    // // Resources::MtnMech,
    // // 10,
    // // vec![],
    // // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),

    // // Operation
    // // 1,
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Resources::MtnMech,

    // let operation_parameter = OperationParameter::new(work_order_number,
    // operation);

    // let mut project_operation_parameters = HashMap::new();
    // project_operation_parameters.insert(1, operation_parameter);

    // let project_work_order_parameter =
    //     ProjectParameter::new(work_order,
    // project_operation_parameters);

    // project_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, project_work_order_parameter);

    // let activity_number = 0;

    // let mut project_activities = ProjectScheduledOperations::default();

    // project_activities.0.insert(
    //     activity_number,
    //     OperationSolution::new(
    //         vec![],
    //         Resources::MtnMech,
    //         1,
    //         Work::from(0.0),
    //         work_order_number,
    //         activity_number,
    //     ),
    // );

    // project_algorithm
    //     .solution
    //     .project_scheduled_work_orders
    //     .0
    //     .insert(
    //         work_order_number,
    //         WhereIsWorkOrder::Project(project_activities),
    //     );

    // project_algorithm.schedule().unwrap();

    // let scheduled_date = project_algorithm
    //     .solution
    //     .project_scheduled_days(&work_order_number, 0);

    // assert!(scheduled_date.is_ok());
}

#[test]
fn test_schedule_2()
{
    // let work_order_number = WorkOrderNumber(2100000010);
    // let activity_number = 1;
    // let first_period = Period::from_str("2024-W13-14").unwrap();

    // let project_days = |number_of_days: u32| -> Vec<Day> {
    //     let mut days: Vec<Day> = Vec::new();
    //     let mut date = first_period.start_date().to_owned();
    //     for day_index in 0..number_of_days {
    //         days.push(Day::new(day_index as usize, date.to_owned()));
    //         date = date.checked_add_days(Days::new(1)).unwrap();
    //     }
    //     days
    // };

    // let id = Id::default();
    // let options = ProjectOptions::default();
    // TODO: Make SchedulingEnvironment testable with dependency injection
    // let scheduling_environment = SchedulingEnvironment::default();
    // let project_parameters = ProjectParameters::new(&id, options,
    // &scheduling_environment)?; let project_solution =
    // ProjectSolution::new(&project_parameters);
    // TODO: Refactor file structure and establish clearer testing patterns
    // let mut project_algorithm = Algorithm::new(
    //     project_days(56),
    //     ProjectResources::new_from_data(
    //         Resources::iter().collect(),
    //         project_days(56),
    //         Work::from(100.0),
    //     ),
    //     ProjectResources::new_from_data(
    //         Resources::iter().collect(),
    //         project_days(56),
    //         Work::from(0.0),
    //     ),
    //     SystemSolution::new(),
    // );

    // let mut project_activities = ProjectScheduledOperations::default();

    // project_activities.0.insert(
    //     activity_number,
    //     OperationSolution::new(
    //         vec![],
    //         Resources::MtnMech,
    //         1,
    //         Work::from(0.0),
    //         work_order_number,
    //         activity_number,
    //     ),
    // );

    // project_algorithm
    //     .solution
    //     .project_scheduled_work_orders
    //     .0
    //     .insert(
    //         work_order_number,
    //         WhereIsWorkOrder::Project(project_activities),
    //     );

    // // Operation
    // // 1,
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Work::from(1.0),
    // // Resources::MtnMech,
    // let operation_parameter = OperationParameter::new(work_order_number,
    // operation);

    // let mut operation_parameters = HashMap::new();
    // operation_parameters.insert(1, operation_parameter);

    // // Work Order
    // // Resources::MtnMech,
    // // 10,
    // // vec![],
    // // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
    // let optimized_project_work_order = ProjectParameter::new(work_order,
    // operation_parameters);

    // project_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, optimized_project_work_order);

    // project_algorithm.schedule().unwrap();

    // let scheduled_date = project_algorithm
    //     .solution
    //     .project_scheduled_days(&work_order_number, 1);

    // assert!(scheduled_date.is_ok());
}
