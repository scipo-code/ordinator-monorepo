use std::fmt::Debug;

use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::time_environment::period::Period;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct WeeklyObjectiveValueResponse
{
    field_one: String,
}

#[derive(Debug, Serialize)]
pub struct WeeklyResponsePeriods
{
    periods: Vec<Period>,
}

impl WeeklyResponsePeriods
{
    pub fn new(periods: Vec<Period>) -> Self
    {
        Self { periods }
    }
}

// TODO: Create a custom type for WeeklyResourcesApi instead of exposing this low-level type
#[derive(Debug, Serialize)]
pub enum WeeklyResponseResources
{
    UpdatedResources(u32),
    LoadingAndCapacities(WeeklyResourcesApi),
    Percentage(WeeklyResourcesApi, WeeklyResourcesApi),
}

#[derive(Debug, Serialize)]
pub struct WeeklyResourcesApi {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyResponseScheduling
{
    work_orders: usize,
    periods: Period,
}

impl WeeklyResponseScheduling
{
    pub fn new(number_of_work_orders_changed: usize, period: Period) -> Self
    {
        Self {
            work_orders: number_of_work_orders_changed,
            periods: period,
        }
    }
}
use ordinator_scheduling_environment::Asset;

use crate::algorithm::strategic_solution::WeeklySolution;
use crate::WeeklyActor;

#[derive(Debug, Serialize)]
pub struct WeeklyResponseStatus
{
    pub asset: Asset,
    pub strategic_objective_value: usize,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl<Ss> From<&mut WeeklyActor<Ss>> for WeeklyResponseStatus
where
    Ss: SystemSolutions<Weekly = WeeklySolution> + Debug,
{
    fn from(value: &mut WeeklyActor<Ss>) -> Self
    {
        let strategic_parameters = &value.algorithm.parameters;

        let number_of_strategic_work_orders =
            strategic_parameters.strategic_work_order_parameters.len();

        let asset = value.actor_id.asset();

        let number_of_periods = value.algorithm.parameters.strategic_periods.len();

        // TODO: Consider creating a trait for objective value extraction instead of direct field access
        WeeklyResponseStatus {
            number_of_strategic_work_orders,
            number_of_periods,
            asset: asset.clone(),
            strategic_objective_value: value.algorithm.solution.objective_value().objective_value
                as usize,
        }
    }
}
