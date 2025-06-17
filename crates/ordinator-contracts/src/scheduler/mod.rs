use ordinator_scheduling_environment::SchedulingEnvironment;

struct SchedulerWorkOrderDto(Vec<SingleRow>);

// This should all be strings. You should reuse the logic from the other
// component. I do not see what other aspect that we have.
struct SingleRow
{
    scheduled_period: Option<Period>,
    scheduled_start_data: Option<Day>,
    priority: Priority,
    revision: Revision,
    work_order_type: WorkOrderType,
    main_work_ctr: Resources,
    operation_work_center: Resources,
    work_order_number: WorkOrderNumber,
    description_work_order: String,
    operation_short_text: String,
    material_status: MaterialStatus,
    system_status: SystemStatusCodes,
    user_status: UserStatusCodes,
    work: Work,
    actual_work: Work,
    unloading_point: UnloadingPoint,
    basic_start_date: DATS,
    basic_finish_date: DATS,
    earliest_start_date: DATS,
    earliest_finish_date: DATS,
    earliest_allowed_start_date: DATS,
    latest_allowed_finish_date: DATS,
    activity: ActivityNumber,
    // operation_system_status: SystemStatusCodes,
    // operation_user_status: SystemStatusCodes,
    functional_location: FunctionalLocation,
    description_operation: String,
    subnetwork_of: String,
    system_condition: SystemCondition,
    maintenance_plan: String,
    planner_group: String,
    maintenance_plant: String,
    pm_collective: String,
    room: String,
}

impl From<(SchedulingEnvironment, TotalSystemSolutions)> for SchedulerWorkOrderDto
{
    fn from(value: (SchedulingEnvironment, TotalSystemSolutions)) -> Self
    {
        todo!()
    }
}
