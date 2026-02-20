use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DailyResponseResources {}

#[derive(Debug, Serialize)]
pub struct DailyResponseScheduling {}

#[derive(Debug, Serialize)]
pub struct DailyResponseStatus
{
    pub delegated_work_order_activities: usize,
    pub objective: u64,
}

#[derive(Debug, Serialize)]
pub struct DailyResponseTime {}
