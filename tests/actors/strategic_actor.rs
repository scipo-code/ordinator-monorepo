    #[test]
    fn test_update_scheduler_state() -> Result<()>
    {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicRequestScheduling::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::builder().build()));

        let system_configuration = SystemConfigurations::read_all_configs().unwrap();
        let strategic_options = StrategicOptions::from((system_configuration, &Id::default()));

        let algorithm: StrategicAlgorithm<Ss> = Algorithm::builder()
            .id(Id::default())
            .parameters(strategic_options, &scheduling_environment.lock().unwrap());

        Ok(())
    }

    #[test]
    fn test_calculate_objective_value() -> Result<()>
    {
        let work_order_number = WorkOrderNumber(2100023841);

        let period = Period::from_str("2023-W49-50").unwrap();

        let operational_resource_1 = OperationalResource::new("OP_TEST_0", Work::from(40.0), vec![
            Resources::MtnMech,
            Resources::MtnElec,
            Resources::VenMech,
        ]);
        let operational_resource_2 = OperationalResource::new("OP_TEST_1", Work::from(40.0), vec![
            Resources::MtnScaf,
            Resources::MtnElec,
            Resources::VenMech,
        ]);
        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        let scheduling_environment = Arc::new(Mutex::new(SchedulingEnvironment::default()));

        let id = Id::default();

        let strategic_options = StrategicOptions::default();

        let mut strategic_parameters = StrategicParameters::new(
            &id,
            strategic_options,
            &scheduling_environment.lock().unwrap(),
        )?;

        let strategic_parameter = WorkOrderParameter::new(
            Some(period),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::from([(Resources::MtnMech, Work::from(10.0))]),
        );

        // TODO: Consider dependency injection for `SchedulingEnvironment` and add insert
        // functions to all types implementing `Parameters`
        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        Ok(())
    }
}
