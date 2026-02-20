// TODO: Move to project
#[test]
#[ignore]
fn test_calculate_objective_value()
{
    let work_order_number = WorkOrderNumber(2100000001);
    let activity_number = 1;
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let project_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };
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
    // SchedulingEnvironment requires arc mutex protection for thread-safe access
    let scheduling_environment = SchedulingEnvironment::builder()
        .work_orders
        .time_environment_builder(|ib| ib.project_days("2025-02-22T07:00:00Z", 56))
        .build();

    // TODO: Add functions to create SchedulingEnvironment

    let id = Id::default();

    let system_configurations = SystemConfigurations::read_all_configs().unwrap().load();

    // Note: This test should be in integration tests with proper SharedSolution initialization
    let algorithm: TacticalAlgorithm = Algorithm::builder()
        .id(id)
        .parameters_and_solution(
            &system_configurations,
            &scheduling_environment.lock().unwrap(),
        )
        .unwrap()
        .build();

    // TODO: Add SystemConfigurations::weekly_options() method
    // TODO: Pass system configuration to Orchestrator and Agents
    // TODO: Add extraction methods to SystemConfiguration

    // let mut project_algorithm = Algorithm::new(
    //     &id,
    //     solution,
    //     parameters,
    //     ArcSwapSharedSolution::default().into(),
    // );

    // // This whole thing is ugly. Remember, you should work on getting the
    // configs // into the program, not the other way around.

    // // FIX
    // // This does not confine to the correct interface setup of the
    // program. You // should think about this in the code. What
    // other thing could you do // here?
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
    //     TacticalParameter::new(&work_order, operation_parameters);

    // project_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, optimized_project_work_order);

    // project_algorithm.calculate_objective_value().unwrap();

    // // assert_eq!(project_algorithm.objective_value().0, 270);
}

// Test setup needs refactoring
#[test]
fn test_schedule_1()
{
    let work_order_number = WorkOrderNumber(2100000001);
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let project_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };

    // TODO: Defer test implementation until system is operational
    // The commented code below will be uncommented and refactored later
    // let mut project_algorithm =
    // Algorithm::builder().new(     project_days(56),
    //     TacticalResources::new_from_data(
    //         Resources::iter().collect(),
    //         project_days(56),
    //         Work::from(0.0),
    //     ),
    //     TacticalResources::new_from_data(
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
    //     TacticalParameter::new(work_order,
    // project_operation_parameters);

    // project_algorithm
    //     .parameters_mut()
    //     .insert(work_order_number, project_work_order_parameter);

    // let activity_number = 0;

    // let mut project_activities = TacticalScheduledOperations::default();

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
    //         WhereIsWorkOrder::Tactical(project_activities),
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
    let work_order_number = WorkOrderNumber(2100000010);
    let activity_number = 1;
    let first_period = Period::from_str("2024-W13-14").unwrap();

    let project_days = |number_of_days: u32| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };

    let id = Id::default();
    let options = TacticalOptions::default();
    let scheduling_environment = SchedulingEnvironment::default();

    // SchedulingEnvironment

    let project_parameters = TacticalParameters::new(&id, options, &scheduling_environment)?;
    let project_solution = TacticalSolution::new(&project_parameters);

    let mut project_algorithm = Algorithm::new(
        project_days(56),
        TacticalResources::new_from_data(
            Resources::iter().collect(),
            project_days(56),
            Work::from(100.0),
        ),
        TacticalResources::new_from_data(
            Resources::iter().collect(),
            project_days(56),
            Work::from(0.0),
        ),
        ArcSwapSharedSolution::default().into(),
    );

    let mut project_activities = TacticalScheduledOperations::default();

    project_activities.0.insert(
        activity_number,
        OperationSolution::new(
            vec![],
            Resources::MtnMech,
            1,
            Work::from(0.0),
            work_order_number,
            activity_number,
        ),
    );

    project_algorithm
        .solution
        .project_scheduled_work_orders
        .0
        .insert(
            work_order_number,
            WhereIsWorkOrder::Tactical(project_activities),
        );

    // Operation
    // 1,
    // Work::from(1.0),
    // Work::from(1.0),
    // Work::from(1.0),
    // Resources::MtnMech,
    let operation_parameter = OperationParameter::new(work_order_number, operation);

    let mut operation_parameters = HashMap::new();
    operation_parameters.insert(1, operation_parameter);

    // Work Order
    // Resources::MtnMech,
    // 10,
    // vec![],
    // NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
    let optimized_project_work_order = TacticalParameter::new(work_order, operation_parameters);

    project_algorithm
        .parameters_mut()
        .insert(work_order_number, optimized_project_work_order);

    project_algorithm.schedule().unwrap();

    let scheduled_date = project_algorithm
        .solution
        .project_scheduled_days(&work_order_number, 1);

    assert!(scheduled_date.is_ok());
}
