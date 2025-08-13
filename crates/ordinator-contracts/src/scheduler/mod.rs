use std::sync::Arc;
use std::sync::MutexGuard;

use anyhow::Result;
use anyhow::anyhow;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::OperationView;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use serde::Serialize;
use tracing::info;
use tracing::warn;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::TotalSystemSolution;
use crate::WorkOrderNumberDto;

#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct SchedulerWorkOrderDto(pub Vec<SingleRowDto>);

#[derive(Serialize, ToSchema, TS, Clone)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
enum PeriodStatus
{
    Frozen,
    Draft,
    Active,
    NotScheduled,
}

impl PeriodStatus
{
    pub fn status_for(period: &Period, frozen_and_draft_periods: &[Period]) -> PeriodStatus
    {
        // NOTE: Is this also correct for other firms or is this Total Specific?
        match period {
            p if *p == frozen_and_draft_periods[0] => PeriodStatus::Frozen,
            p if *p == frozen_and_draft_periods[1] => PeriodStatus::Draft,
            _ => PeriodStatus::Active,
        }
    }
}

// This should all be strings. You should reuse the logic from the other
// component. I do not see what other aspect that we have.
#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
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

impl SingleRowDto
{
    pub fn get_suggested_period(&self) -> String
    {
        self.suggested_scheduled_period.to_string()
    }
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
            .filter(|(_, wo)| wo.functional_location().asset == value.0)
            .map(|(_, wo)| wo)
            .collect();

        let periods = value.1.time_environment.periods.clone();
        let periods_for_frozen_and_draft = [periods[0].clone(), periods[1].clone()];

        info!(target: "developer", tactical_work_orders = system_solution.tactical.clone().unwrap().all_scheduled_tasks().len());
        let time = std::time::Instant::now();
        warn!(target: "developer", time_first = ?time);
        for work_order in work_orders_by_asset {
            let work_order_view = work_order.view();
            let sorted_operations = work_order_view.operations.iter().collect::<Vec<_>>();

            let strategic_period = system_solution
                .strategic()?
                .scheduled_task(&work_order_view.work_order_number);

            let strategic_schedule = match strategic_period {
                Some(opt_period) => match opt_period {
                    WhereIsWorkOrder::Strategic(period) => period.clone().to_string(),
                    // This does not have to be perfect.
                    WhereIsWorkOrder::Tactical(period) => period.clone().to_string(),
                    WhereIsWorkOrder::NotScheduled => {
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
                    WhereIsWorkOrder::Strategic(period) => {
                        PeriodStatus::status_for(period, &periods_for_frozen_and_draft)
                    }
                    WhereIsWorkOrder::Tactical(period) => {
                        PeriodStatus::status_for(period, &periods_for_frozen_and_draft)
                    }
                    WhereIsWorkOrder::NotScheduled => PeriodStatus::NotScheduled,
                },
                None => PeriodStatus::NotScheduled,
            };

            let tactical_solution = &system_solution
                .tactical
                .as_ref()
                .ok_or(anyhow!("There is no TacticalActor present"))?;

            for operation_view in sorted_operations {
                let tactical_days = tactical_solution.start_and_finish_dates(&(
                    work_order_view.work_order_number,
                    operation_view.activity,
                ));
                let option_day = match tactical_days {
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
                    work_order_number: work_order_view.work_order_number.to_string(),
                    suggested_scheduled_period: strategic_schedule.clone(),
                    scheduled_start_date: option_day.to_string(),
                    period_status: period_status.clone(),
                    priority: work_order_view.priority.to_string(),
                    revision: work_order_view.revision.to_string(),
                    work_order_type: work_order_view.work_order_type.to_string(),
                    main_work_ctr: work_order_view.main_work_ctr.to_string(),
                    operation_work_center: operation_view.resource.to_string(),
                    description_work_order: work_order_view.description_work_order.clone(),
                    operation_short_text: operation_view.description_operation.to_string(),
                    material_status: work_order_view.material_status.to_string(),
                    system_status: work_order_view.system_status.to_string(),
                    user_status: work_order_view.user_status.to_string(),
                    work: operation_view.remaining_work.to_string(),
                    actual_work: operation_view.actual_work.to_string(),
                    unloading_point: operation_view.unloading_point.to_string(),
                    basic_start_date: work_order_view.basic_start_date.to_string(),
                    basic_finish_date: work_order_view.basic_finish_date.to_string(),
                    earliest_start_date: operation_view.earliest_start_datetime.to_string(),
                    earliest_finish_date: operation_view.earliest_finish_datetime.to_string(),
                    earliest_allowed_start_date: work_order_view
                        .earliest_allowed_start_date
                        .to_string(),
                    latest_allowed_finish_date: work_order_view
                        .latest_allowed_finish_date
                        .to_string(),
                    activity: operation_view.activity.to_string(),
                    // operation_system_status: work_order_view.status_codes().clone(),
                    // operation_user_status: work_order_view.status_codes().clone(),
                    functional_location: work_order_view.functional_location.to_string(),
                    description_operation: operation_view.description_operation.to_string(),
                    subnetwork_of: work_order_view.subnetwork_of.clone(),
                    system_condition: work_order_view.system_condition.to_string(),
                    maintenance_plan: work_order_view.maintenance_plan.clone(),
                    planner_group: work_order_view.planner_group.clone(),
                    maintenance_plant: work_order_view.maintenance_plant.clone(),
                    pm_collective: work_order_view.pm_collective.clone(),
                    room: work_order_view.room.clone(),
                };

                all_rows.push(one_row);
            }
        }

        warn!(target: "developer", time = ?time.elapsed());
        Ok(SchedulerWorkOrderDto(all_rows))
    }
}

type ResourcesDto = String;

#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct WorkOrderSingleRowSimpleDto
{
    work_order_number: u64,
    main_work_center: ResourcesDto,
    operations: Vec<OperationDto>,
    functional_location: String,
    sch: bool,
    awsc: bool,
    vendor: bool,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct WorkOrderInfoWithSchedulingDto
{
    asset: String,
    work_order_number: u64,
    main_work_center: ResourcesDto,
    operations: Vec<OperationDto>,
    functional_location: String,
    sch: bool,
    awsc: bool,
    vendor: bool,
    priority: String,
    revision: String,
    period_status: PeriodStatus,
    suggested_scheduled_period: String,
    basic_start_date: String,
    basic_finish_date: String,
}

impl
    TryFrom<(
        Asset,
        MutexGuard<'_, SchedulingEnvironment>,
        arc_swap::Guard<Arc<TotalSystemSolution>>,
        WorkOrderNumberDto,
    )> for WorkOrderInfoWithSchedulingDto
{
    type Error = anyhow::Error;

    fn try_from(
        value: (
            Asset,
            MutexGuard<'_, SchedulingEnvironment>,
            arc_swap::Guard<Arc<TotalSystemSolution>>,
            WorkOrderNumberDto,
        ),
    ) -> Result<Self>
    {
        let work_order_number_requested = WorkOrderNumber::from(value.3);
        let work_order = value.1.work_orders.inner.get(&work_order_number_requested);

        let work_order = match work_order {
            Some(wo) => wo.clone(),
            None => {
                return Err(anyhow!(
                    "Work order {} does not exist",
                    work_order_number_requested
                ));
            }
        };

        let work_order_view = work_order.view();

        let system_solution = value.2;

        let strategic_period = system_solution
            .strategic()?
            .scheduled_task(&work_order.work_order_number());

        let strategic_schedule = match strategic_period {
            Some(opt_period) => match opt_period {
                WhereIsWorkOrder::Strategic(period) => period.clone().to_string(),
                // This does not have to be perfect.
                WhereIsWorkOrder::Tactical(period) => period.clone().to_string(),
                WhereIsWorkOrder::NotScheduled => {
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

        let periods = value.1.time_environment.periods.clone();
        let periods_for_frozen_and_draft = [periods[0].clone(), periods[1].clone()];
        let period_status = match strategic_period {
            Some(opt_period) => match opt_period {
                WhereIsWorkOrder::Strategic(period) => {
                    PeriodStatus::status_for(period, &periods_for_frozen_and_draft)
                }
                WhereIsWorkOrder::Tactical(period) => {
                    PeriodStatus::status_for(period, &periods_for_frozen_and_draft)
                }
                WhereIsWorkOrder::NotScheduled => PeriodStatus::NotScheduled,
            },
            None => PeriodStatus::NotScheduled,
        };

        let operations_with_dates: Result<Vec<OperationDto>, anyhow::Error> = work_order_view
            .operations
            .iter()
            .map(|operation| {
                let tactical_date = &system_solution
                    .tactical
                    .as_ref()
                    .ok_or(anyhow!("There is no TacticalAgent present"))?
                    .start_and_finish_dates(&(
                        work_order_view.work_order_number,
                        operation.activity,
                    ));

                let scheduled_date = match tactical_date {
                    Some(date) => date.0.to_string(),
                    None => "No scheduled start date".to_string(),
                };

                let ac =
                    OperationDto::from(operation.clone()).add_scheduled_start_date(scheduled_date);
                Ok(ac)
            })
            .collect();

        let operations = operations_with_dates?;

        let work_order_info = WorkOrderInfoWithSchedulingDto {
            asset: value.0.to_string(),
            work_order_number: work_order_view.work_order_number.0,
            main_work_center: work_order_view.main_work_ctr.to_string(),
            operations,
            functional_location: work_order_view.functional_location.to_string(),
            sch: work_order_view.user_status.sch,
            awsc: work_order_view.user_status.awsc,
            vendor: work_order_view.vendor,
            priority: work_order_view.priority.to_string(),
            revision: work_order_view.revision.to_string(),
            period_status,
            suggested_scheduled_period: strategic_schedule.clone(),
            basic_start_date: work_order_view.basic_start_date.to_string(),
            basic_finish_date: work_order_view.basic_finish_date.to_string(),
        };

        Ok(work_order_info)
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
struct OperationDto
{
    activity: u64,
    work_remaining: f64,

    work_center: ResourcesDto,
    number_of_people: u64,
    unloading_point_string: String,
    scheduled_start_date: Option<String>,
}

impl From<WorkOrder> for WorkOrderSingleRowSimpleDto
{
    fn from(value: WorkOrder) -> Self
    {
        let work_order_view = value.view();
        Self {
            work_order_number: work_order_view.work_order_number.0,
            main_work_center: work_order_view.main_work_ctr.to_string(),
            operations: work_order_view
                .operations
                .iter()
                .map(|e| OperationDto::from((*e).clone()))
                .collect(),
            functional_location: work_order_view.functional_location.to_string(),
            sch: work_order_view.user_status.sch,
            awsc: work_order_view.user_status.awsc,
            vendor: work_order_view.vendor,
        }
    }
}

impl From<OperationView> for OperationDto
{
    fn from(value: OperationView) -> Self
    {
        Self {
            activity: value.activity,
            work_remaining: value.remaining_work.to_f64(),
            work_center: value.resource.to_string(),
            number_of_people: value.number_of_people,
            unloading_point_string: value.unloading_point,
            scheduled_start_date: None,
        }
    }
}

impl OperationDto
{
    pub fn add_scheduled_start_date(mut self, date: String) -> Self
    {
        self.scheduled_start_date = Some(date);
        self
    }
}
