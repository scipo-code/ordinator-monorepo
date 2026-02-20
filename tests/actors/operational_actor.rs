#[test]
fn test_determine_first_available_start_time() -> Result<()>
{
    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();

    let operational_algorithm: OperationalAlgorithm<SharedSolution> = Algorithm::builder()
        .id(id)
        .parameters(&scheduling_environment)?
        .solution()
        .build();

    operational_algorithm.load_shared_solution();

    let mut strategic_updated_shared_solution =
        (**operational_algorithm.loaded_shared_solution).clone();

    // Isolate strategic changes to avoid coupling test setup directly to actors
    strategic_updated_shared_solution
        .strategic
        .strategic_scheduled_work_orders
        .insert(
            WorkOrderNumber(0),
            Some(Period::from_str("2024-W41-42").unwrap()),
        );

    operational_algorithm
        .arc_swap_shared_solution
        .0
        .store(Arc::new(strategic_updated_shared_solution));

    operational_algorithm.load_shared_solution();
    let mut project_updated_shared_solution =
        (**operational_algorithm.loaded_shared_solution).clone();

    project_updated_shared_solution
        .project
        .project_work_orders
        .0
        .insert(WorkOrderNumber(0), WhereIsWorkOrder::NotScheduled);

    operational_algorithm
        .arc_swap_shared_solution
        .0
        .store(Arc::new(project_updated_shared_solution));

    operational_algorithm.load_shared_solution();

    let operational_parameter = OperationalParameter::new(Work::from(20.0), Work::from(0.0))
        .expect("Work has to be non-zero to create an OperationalParameter");

    let start_time = operational_algorithm
        .determine_first_available_start_time(&(WorkOrderNumber(0), 0), &operational_parameter)
        .unwrap();

    assert_eq!(
        start_time,
        DateTime::parse_from_rfc3339("2024-10-07T08:00:00Z")
            .unwrap()
            .to_utc()
    );
    Ok(())
}
#[test]
fn test_determine_next_event_3() -> Result<()>
{
    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();

    let value = SystemConfigurations::read_all_configs().unwrap();

    let operational_algorithm: OperationalAlgorithm<SharedSolution> = Algorithm::builder()
        .id(id)
        .parameters(&scheduling_environment)?
        .solution()
        .build();

    let current_time = DateTime::parse_from_rfc3339("2024-05-20T01:00:00Z")
        .unwrap()
        .to_utc();

    let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

    assert_eq!(time_delta, TimeDelta::new(3600 * 6, 0).unwrap());
    // TODO: Add assertion for next_event once OperationalEvents::Toolbox API is finalized
    Ok(())
}
#[test]
fn test_determine_next_event_2() -> Result<()>
{
    // TODO: Refactor determine_next_event as a data function instead of algorithm method.
    // This allows computing event timing without full algorithm context and improves testability.
    Ok(())
}
#[test]
fn test_determine_next_event_1() -> Result<()>
{
    let system_configurations = SystemConfigurations::read_all_configs()?;

    let mut scheduling_environment = SchedulingEnvironment::builder().build();

    let id = Id::new("TEST_OPERATIONAL", vec![], vec![]);

    // TODO: Initialize operational agents from central configuration instead of inline setup
    scheduling_environment
        .worker_environment
        .agent_environment
        .operational
        .insert(id.clone(), operational_configuration_all);

    let scheduling_environment = Arc::new(Mutex::new(scheduling_environment)).lock().unwrap();

    let operational_algorithm: OperationalAlgorithm<Ss> = Algorithm::builder()
        .id(id)
        // Parameters do not need the `options`
        .parameters(&scheduling_environment)?
        .solution()
        .build();

    let current_time = DateTime::parse_from_rfc3339("2024-05-20T12:00:00Z")
        .unwrap()
        .to_utc();

    let (time_delta, next_event) = operational_algorithm.determine_next_event(&current_time);

    assert_eq!(time_delta, TimeDelta::new(3600 * 7, 0).unwrap());

    assert_eq!(next_event, OperationalEvents::OffShift(off_shift_interval));
    Ok(())
}
