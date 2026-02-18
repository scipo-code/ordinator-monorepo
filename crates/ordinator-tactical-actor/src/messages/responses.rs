use ordinator_scheduling_environment::time_environment::day::Day;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponseScheduling {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponseStatus
{
    objective: u64,
    time_horizon: Vec<Day>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponseTime {}

#[derive(Debug, Serialize)]
pub struct ProjectResponseUpdate {}
