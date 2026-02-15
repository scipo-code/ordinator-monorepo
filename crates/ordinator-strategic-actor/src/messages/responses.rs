use std::fmt::Debug;

use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::time_environment::period::Period;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct StrategicObjectiveValueResponse
{
    field_one: String,
}

#[derive(Debug, Serialize)]
pub struct StrategicResponsePeriods
{
    periods: Vec<Period>,
}

impl StrategicResponsePeriods
{
    pub fn new(periods: Vec<Period>) -> Self
    {
        Self { periods }
    }
}

// TODO: Create a custom type for StrategicResourcesApi instead of exposing this low-level type
#[derive(Debug, Serialize)]
pub enum StrategicResponseResources
{
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResourcesApi),
    Percentage(StrategicResourcesApi, StrategicResourcesApi),
}

#[derive(Debug, Serialize)]
pub struct StrategicResourcesApi {}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategicResponseScheduling
{
    work_orders: usize,
    periods: Period,
}

impl StrategicResponseScheduling
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

use crate::algorithm::strategic_solution::StrategicSolution;
use crate::StrategicActor;

#[derive(Debug, Serialize)]
pub struct StrategicResponseStatus
{
    pub asset: Asset,
    pub strategic_objective_value: usize,
    pub number_of_strategic_work_orders: usize,
    pub number_of_periods: usize,
}

impl<Ss> From<&mut StrategicActor<Ss>> for StrategicResponseStatus
where
    Ss: SystemSolutions<Strategic = StrategicSolution> + Debug,
{
    fn from(value: &mut StrategicActor<Ss>) -> Self
    {
        let strategic_parameters = &value.algorithm.parameters;

        let number_of_strategic_work_orders =
            strategic_parameters.strategic_work_order_parameters.len();

        let asset = value.actor_id.asset();

        let number_of_periods = value.algorithm.parameters.strategic_periods.len();

        // TODO: Consider creating a trait for objective value extraction instead of direct field access
        StrategicResponseStatus {
            number_of_strategic_work_orders,
            number_of_periods,
            asset: asset.clone(),
            strategic_objective_value: value.algorithm.solution.objective_value().objective_value
                as usize,
        }
    }
}
