// use std::str::FromStr;
// use std::sync::Arc;
// use std::sync::Mutex;

// use ordinator_scheduling_environment::SchedulingEnvironment;
// use ordinator_scheduling_environment::time_environment::period::Period;
// use ordinator_scheduling_environment::work_order::WorkOrderNumber;
// use ordinator_scheduling_environment::work_order::operation::Work;
// use ordinator_scheduling_environment::worker_environment::resources::Resources;
// use ordinator_weekly_actor::algorithm::weekly_resources::OperationalResource;
// use ordinator_weekly_actor::algorithm::weekly_resources::WeeklyResources;

#[test]
fn test_update_scheduler_state() -> anyhow::Result<()>
{
    // let work_order_number = WorkOrderNumber(2200002020);
    // let vec_work_order_number = vec![work_order_number];
    // let period_string: String = "2023-W47-48".to_string();

    // let schedule_work_order = ScheduleChange::new(vec_work_order_number,
    // period_string);

    // let weekly_scheduling_internal =
    // WeeklyRequestScheduling::Schedule(schedule_work_order);

    // let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

    // let scheduling_environment =
    // Arc::new(Mutex::new(SchedulingEnvironment::builder().build()));

    // let system_configuration = SystemConfigurations::read_all_configs().unwrap();
    // let weekly_options = WeeklyOptions::from((system_configuration,
    // &Id::default()));

    // let algorithm: WeeklyAlgorithm<Ss> = Algorithm::builder()
    //     .id(Id::default())
    //     .parameters(weekly_options, &scheduling_environment.lock().unwrap());

    // let mut weekly_algorithm = Algorithm::new(
    //     &id,
    //     weekly_solution,
    //     weekly_parameters,
    //     ArcSwapSharedSolution::default().into(),
    // );

    // let weekly_parameter = WorkOrderParameter::new(
    //     Some(periods[0].clone()),
    //     HashSet::new(),
    //     periods.first().unwrap().clone(),
    //     1000,
    //     HashMap::new(),
    // );

    // weekly_algorithm
    //     .parameters
    //     .weekly_work_order_parameters
    //     .insert(work_order_number, weekly_parameter);

    // weekly_algorithm
    //     .update_scheduling_state(weekly_scheduling_internal)
    //     .unwrap();

    // assert_eq!(
    //     weekly_algorithm
    //         .parameters
    //         .weekly_work_order_parameters
    //         .get(&work_order_number)
    //         .as_ref()
    //         .unwrap()
    //         .locked_in_period
    //         .as_ref()
    //         .unwrap()
    //         .period_string(),
    //     "2023-W47-48"
    // );
    Ok(())
}

#[test]
fn test_calculate_objective_value() -> anyhow::Result<()>
{
    // let work_order_number = WorkOrderNumber(2100023841);

    // let period = Period::from_str("2023-W49-50").unwrap();

    // let operational_resource_1 = OperationalResource::new(
    //     "OP_TEST_0",
    //     Work::from(40.0),
    //     vec![Resources::MtnMech, Resources::MtnElec, Resources::VenMech],
    // );
    // let operational_resource_2 = OperationalResource::new(
    //     "OP_TEST_1",
    //     Work::from(40.0),
    //     vec![Resources::MtnScaf, Resources::MtnElec, Resources::VenMech],
    // );
    // let mut weekly_resources = WeeklyResources::default();

    // weekly_resources.insert_operational_resource(period.clone(),
    // operational_resource_1); weekly_resources.
    // insert_operational_resource(period.clone(), operational_resource_2);

    // let scheduling_environment =
    // Arc::new(Mutex::new(SchedulingEnvironment::default()));

    // TODO: All functions need testing, but currently blocked by Id::default() issue

    // let weekly_options = WeeklyOptions::default();

    // let mut weekly_parameters = WeeklyParameters::new(
    //     &id,
    //     weekly_options,
    //     &scheduling_environment.lock().unwrap(),
    // )?;

    // let weekly_parameter = WorkOrderParameter::new(
    //     Some(period),
    //     HashSet::new(),
    //     Period::from_str("2023-W47-48").unwrap(),
    //     1000,
    //     HashMap::from([(Resources::MtnMech, Work::from(10.0))]),
    // );

    // TODO: Consider dependency injection for SchedulingEnvironment to allow
    // all Parameters implementers to have an insert function
    // weekly_parameters
    //     .insert_weekly_parameter(WorkOrderNumber(2100023841),
    // weekly_parameter);

    // // let weekly_solution =
    // WeeklySolution::new(&weekly_parameters); // let mut
    // weekly_algorithm = Algorithm::new( //     &Id::default(),
    // //     weekly_solution,
    // //     weekly_parameters,
    // //     ArcSwapSharedSolution::default().into(),
    // // );

    // // weekly_algorithm
    // //     .solution
    // //     .weekly_scheduled_work_orders
    // //     .insert(work_order_number, None);

    // // weekly_algorithm
    // //     .schedule_forced_work_order(&
    // ForcedWorkOrder::Locked(work_order_number))? // ;

    // // let objective_value_type =
    // weekly_algorithm.calculate_objective_value()?;

    // // let objective_value =
    // //     if let ObjectiveValueType::Better(objective_value) =
    // objective_value_type // {         objective_value
    // //     } else {
    // //         panic!();
    // //     };

    // // weekly_algorithm.solution.objective_value = objective_value;

    // // assert_eq!(
    // //     weekly_algorithm.solution.objective_value.objective_value,
    // 2000, //     "{:#?}",
    // //     weekly_algorithm.solution.objective_value
    // // );
    Ok(())
}
// use std::fmt::Display;

// TODO: Implement generic Display for Agent to enable viewing all agent types
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "SchedulerAgent: \n
//             Platform: {}, \n",
//             self.asset,
//         )
//     }
// }
