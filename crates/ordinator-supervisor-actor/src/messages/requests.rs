use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailySchedulingMessage
{
    pub work_order_activity: WorkOrderActivity,
    pub id_operational: ActorCompositeId,
}

impl DailySchedulingMessage
{
    pub fn new(work_order_activity: WorkOrderActivity, id_operational: ActorCompositeId) -> Self
    {
        Self {
            work_order_activity,
            id_operational,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DailyStatusMessage
{
    General,
}
pub enum DailyRequestScheduling {}
pub enum DailyRequestResource {}
pub enum DailyTimeRequest {}
pub enum DailySchedulingEnvironmentCommands {}
