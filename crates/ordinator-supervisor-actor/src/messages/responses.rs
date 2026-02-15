use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SupervisorResponseResources {}

#[derive(Debug, Serialize)]
pub struct SupervisorResponseScheduling {}

#[derive(Debug, Serialize)]
pub struct SupervisorResponseStatus
{
    pub delegated_work_order_activities: usize,
    pub objective: u64,
}

#[derive(Debug, Serialize)]
pub struct SupervisorResponseTime {}
