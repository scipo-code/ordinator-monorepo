use std::collections::HashMap;
use std::collections::HashSet;

use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::SystemStatusCodes;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::UserStatusCodes;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfo;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use serde::Serialize;
use ts_rs::TS;

use crate::AssetNames;

// TODO: Move route handlers into the Orchestrator struct.

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum OrchestratorResponse
{
    WorkOrderStatus(WorkOrdersStatus),
    RequestStatus(String),
    Periods(Vec<Period>),
    Days(Vec<Day>),
    Export(String),
    Success,
    Todo,
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum WorkOrdersStatus
{
    Single(WorkOrderResponse),
    SingleSolution(WeeklyApiSolution),
    Multiple(HashMap<WorkOrderNumber, WorkOrderResponse>),
}

#[derive(Serialize)]
pub struct WorkOrderResponse
{
    earliest_period: Period,
    work_order_info: WorkOrderInfo,
    vendor: bool,
    weight: u64,
    work_order_work_load: HashMap<Skill, Work>,
    system_status_codes: SystemStatusCodes,
    user_status_codes: UserStatusCodes,
    api_solution: ApiSolution,
}

#[derive(Serialize)]
pub struct ApiSolution
{
    pub strategic: String,   // TODO: Replace with ApiWeekly type
    pub project: String,    // TODO: Replace with ApiProject type
    pub supervisor: String,  // TODO: Replace with HashMap<Id, ApiDaily>
    pub operational: String, // TODO: Replace with HashMap<Id, ApiOperational>
}

#[derive(Serialize)]
pub struct WeeklyApiSolution
{
    pub solution: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_period: HashSet<Period>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiWeekly
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiProject
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiDaily
{
    solution_data: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ApiOperational
{
    solution_data: String,
}

// TODO: Delete this type and move it to the `conversions` crate.

#[derive(Serialize)]
pub struct OptimizedWorkOrderResponse
{
    scheduled_period: Period,
    locked_in_period: Option<Period>,
    excluded_periods: HashSet<Period>,
    latest_period: Period,
}

impl OptimizedWorkOrderResponse
{
    pub fn new(
        scheduled_period: Period,
        locked_in_period: Option<Period>,
        excluded_periods: HashSet<Period>,
        latest_period: Period,
    ) -> Self
    {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_periods,
            latest_period,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrchestratorMessage<T>
{
    pub message_from_orchestrator: T,
}

impl<T> OrchestratorMessage<T>
{
    pub fn new(message_from_orchestrator: T) -> Self
    {
        Self {
            message_from_orchestrator,
        }
    }
}

#[derive(TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct AvailableAssets
{
    pub assets: Vec<AssetNames>,
}
