use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use serde::Deserialize;
use serde::Serialize;

// HTTP GET and POST endpoints for resource management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectResourceRequest
{
    GetLoadings
    {
        days_end: String,
        select_resources: Option<Vec<Skill>>,
    },
    GetCapacities
    {
        days_end: String,
        select_resources: Option<Vec<Skill>>,
    },
    GetPercentageLoadings
    {
        days_end: String,
        resources: Option<Vec<Skill>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectSchedulingRequest
{
    Schedule(ScheduleChange),
    ScheduleMultiple(Vec<ScheduleChange>),
    ExcludeFromDay(ScheduleChange),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectStatusMessage
{
    General,
    Day(String),
}
#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum ProjectTimeRequest
{
    Days,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectUpdateRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleChange
{
    pub work_order_number: Vec<WorkOrderNumber>,
    pub period_string: String,
}

impl ScheduleChange
{
    pub fn new(work_order_number: Vec<WorkOrderNumber>, period_string: String) -> Self
    {
        Self {
            work_order_number,
            period_string,
        }
    }

    pub fn period_string(&self) -> String
    {
        self.period_string.clone()
    }
}

pub enum ProjectRequestScheduling {}
pub enum ProjectRequestResource {}
pub enum ProjectSchedulingEnvironmentCommands {}
