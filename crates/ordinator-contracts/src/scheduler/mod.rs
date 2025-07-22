use std::sync::Arc;
use std::sync::MutexGuard;

use anyhow::Result;
use anyhow::anyhow;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::MaterialStatus;
use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::PeriodDto;
use crate::TotalSystemSolution;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct SchedulerWorkOrderDto(Vec<SingleRowDto>);

#[derive(Serialize, ToSchema, TS, Clone)]
#[ts(export)]
enum PeriodStatus
{
    Frozen,
    Draft,
    Active,
    NotScheduled,
}

impl PeriodStatus
{
    pub fn status_for(period: &Period, periods: &[Period]) -> PeriodStatus
    {
        // NOTE: Is this also correct for other firms or is this Total Specific?
        match period {
            p if *p == periods[0] => PeriodStatus::Frozen,
            p if *p == periods[1] => PeriodStatus::Draft,
            _ => PeriodStatus::Active,
        }
    }
}

// This should all be strings. You should reuse the logic from the other
// component. I do not see what other aspect that we have.
#[derive(Serialize, ToSchema, TS)]
pub struct SingleRowDto
{
    suggested_scheduled_period: String,
    scheduled_start_date: String,
    period_status: PeriodStatus,
    priority: String,
    revision: String,
    work_order_type: String,
    main_work_ctr: String,
    operation_work_center: String,
    work_order_number: String,
    description_work_order: String,
    operation_short_text: String,
    material_status: String,
    system_status: String,
    user_status: String,
    work: String,
    actual_work: String,
    unloading_point: String,
    basic_start_date: String,
    basic_finish_date: String,
    earliest_start_date: String,
    earliest_finish_date: String,
    earliest_allowed_start_date: String,
    latest_allowed_finish_date: String,
    activity: String,
    // operation_system_status: String,
    // operation_user_status: String,
    functional_location: String,
    description_operation: String,
    subnetwork_of: String,
    system_condition: String,
    maintenance_plan: String,
    planner_group: String,
    maintenance_plant: String,
    pm_collective: String,
    room: String,
}

impl
    TryFrom<(
        Asset,
        MutexGuard<'_, SchedulingEnvironment>,
        arc_swap::Guard<Arc<TotalSystemSolution>>,
    )> for SchedulerWorkOrderDto
{
    type Error = anyhow::Error;

    fn try_from(
        value: (
            Asset,
            MutexGuard<'_, SchedulingEnvironment>,
            arc_swap::Guard<Arc<TotalSystemSolution>>,
        ),
    ) -> Result<Self>
    {
        let mut all_rows: Vec<SingleRowDto> = Vec::new();

        let system_solution = value.2;

        let work_orders_by_asset: Vec<WorkOrder> = value
            .1
            .work_orders
            .clone()
            .inner
            .into_iter()
            .filter(|(_, wo)| wo.work_order_info.functional_location.asset == value.0)
            .map(|(_, wo)| wo)
            .collect();

        let periods = value.1.time_environment.periods.clone();
        let periods_for_frozen_and_draft = [periods[0].clone(), periods[1].clone()];

        for work_order in work_orders_by_asset {
            let sorted_operations = work_order.operations.0.iter().collect::<Vec<_>>();

            let strategic_period = system_solution
                .strategic()?
                .scheduled_task(&work_order.work_order_number);

            let strategic_schedule = match strategic_period {
                Some(opt_period) => match opt_period {
                    Some(period) => period.clone().to_string(),
                    // This does not have to be perfect.
                    None => {
                        "Could not be scheduled under current business rules".to_string()
                        // ReasonForNotScheduling::Unknown(
                        //                     "Strategic Algorithm could
                        // not schedule the Work Order. If this is a mistake
                        // please not down why, and send
                        // a message to
                        // christian-brunbjerg.jespersen@external.totalenergies.
                        // com"
                        // .to_string(),
                        // )
                    }
                },
                None => "Work Order not part of scheduling process".to_string(),
            };

            let period_status = match strategic_period {
                Some(opt_period) => match opt_period {
                    Some(period) => {
                        PeriodStatus::status_for(&period, &periods_for_frozen_and_draft)
                    }
                    None => PeriodStatus::NotScheduled,
                },
                None => PeriodStatus::NotScheduled,
            };

            for activity in sorted_operations {
                let tactical_solution = &system_solution
                    .tactical
                    .as_ref()
                    .ok_or(anyhow!("There is no TacticalActor present"))?
                    .start_and_finish_dates(&(work_order.work_order_number, *activity.0));

                let option_day = match tactical_solution {
                    // Day::index is a weird thing. How should it work in a real time system? I
                    // think that the best approach would
                    Some(tactical_day) => tactical_day.0.to_string(),
                    None => "Work order was not scheduled".to_string(),
                };

                // The issue with what you are doing is that we can keep implementing stuff like
                // this until the day we die. You have to simply go for the money here. I think
                // that is the best approach.
                //
                // You are not good enough to code. You are good enough to do this, Brian
                // believes in you. You simply have to keep working.
                let one_row = SingleRowDto {
                    suggested_scheduled_period: strategic_schedule.clone(),
                    scheduled_start_date: option_day.to_string(),
                    period_status: period_status.clone(),
                    priority: work_order.work_order_info.priority.to_string(),
                    revision: work_order.work_order_info.revision.to_string(),
                    work_order_type: work_order.work_order_info.work_order_type.to_string(),
                    main_work_ctr: work_order.main_work_center.to_string(),
                    operation_work_center: activity.1.resource.to_string(),
                    work_order_number: work_order.work_order_number.to_string(),
                    description_work_order: work_order
                        .work_order_info
                        .work_order_text
                        .order_description
                        .clone(),
                    operation_short_text: work_order
                        .work_order_info
                        .work_order_text
                        .operation_description
                        .clone()
                        .unwrap_or("WE DO NOT HAVE THIS FIELD FROM SAP YET".to_string()),
                    material_status: MaterialStatus::from(
                        work_order.work_order_analytic.user_status_codes.clone(),
                    )
                    .to_string(),
                    system_status: work_order
                        .work_order_analytic
                        .system_status_codes
                        .to_string(),
                    user_status: work_order
                        .work_order_analytic
                        .user_status_codes
                        .clone()
                        .to_string(),
                    work: activity.1.operation_info.work_remaining.to_string(),
                    actual_work: activity.1.operation_info.work_actual.to_string(),
                    unloading_point: activity.1.unloading_point.to_string(),
                    basic_start_date: work_order.work_order_dates.basic_start_date.to_string(),
                    basic_finish_date: work_order.work_order_dates.basic_finish_date.to_string(),
                    earliest_start_date: activity
                        .1
                        .operation_dates
                        .earliest_start_datetime
                        .date_naive()
                        .to_string(),
                    earliest_finish_date: activity
                        .1
                        .operation_dates
                        .earliest_finish_datetime
                        .date_naive()
                        .to_string(),
                    earliest_allowed_start_date: work_order
                        .work_order_dates
                        .earliest_allowed_start_date
                        .to_string(),
                    latest_allowed_finish_date: work_order
                        .work_order_dates
                        .latest_allowed_finish_date
                        .to_string(),
                    activity: activity.0.to_string(),
                    // operation_system_status: work_order.status_codes().clone(),
                    // operation_user_status: work_order.status_codes().clone(),
                    functional_location: work_order.functional_location().to_string(),
                    description_operation: work_order
                        .work_order_info
                        .work_order_text
                        .operation_description
                        .clone()
                        .unwrap_or("WE DO NOT HAVE THIS SAP FIELD YET".to_string()),
                    subnetwork_of: work_order
                        .work_order_info
                        .work_order_info_detail
                        .subnetwork
                        .clone(),
                    system_condition: work_order.work_order_info.system_condition.to_string(),
                    maintenance_plan: work_order
                        .work_order_info
                        .work_order_info_detail
                        .maintenance_plan
                        .clone(),
                    planner_group: work_order
                        .work_order_info
                        .work_order_info_detail
                        .planner_group
                        .clone(),
                    maintenance_plant: work_order
                        .work_order_info
                        .work_order_info_detail
                        .maintenance_plant
                        .clone(),
                    pm_collective: work_order
                        .work_order_info
                        .work_order_info_detail
                        .pm_collective
                        .clone(),
                    room: work_order
                        .work_order_info
                        .work_order_info_detail
                        .room
                        .clone(),
                };

                all_rows.push(one_row);
            }
        }

        Ok(SchedulerWorkOrderDto(all_rows))
    }
}

type ResourcesDto = String;

#[derive(Serialize, ToSchema)]
pub struct WorkOrderSingleRowSimpleDto
{
    work_order_number: u64,
    main_work_center: ResourcesDto,
    operations: Vec<OperationDto>,
    functional_location: String,
}

#[derive(Serialize, ToSchema)]
struct OperationDto
{
    activity: u64,
    work_remaining: f64,

    work_center: ResourcesDto,
    number_of_people: u64,
    unloading_point_period: Option<PeriodDto>,
    unloading_point_string: String,
}

impl From<WorkOrder> for WorkOrderSingleRowSimpleDto
{
    fn from(value: WorkOrder) -> Self
    {
        Self {
            work_order_number: value.work_order_number.0,
            main_work_center: value.main_work_center.to_string(),
            operations: value
                .operations
                .0
                .iter()
                .map(|e| OperationDto::from(e.1.clone()))
                .collect(),
            functional_location: value.functional_location().to_string(),
        }
    }
}

impl From<Operation> for OperationDto
{
    fn from(value: Operation) -> Self
    {
        Self {
            activity: value.activity,
            work_remaining: value.operation_info.work_remaining.to_f64(),
            work_center: value.resource.to_string(),
            number_of_people: value.operation_info.number,
            unloading_point_period: if value.unloading_point.period_string.is_some() {
                Some(PeriodDto(value.unloading_point.period_string.unwrap()))
            } else {
                None
            },
            unloading_point_string: value.unloading_point.string,
        }
    }
}
