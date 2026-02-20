use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Result;
use colored::Colorize;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::WeeklyInterface;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::Percent;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::WeeklyOptions;
use serde::Deserialize;
use serde::Serialize;
use valuable::Valuable;

use super::strategic_parameters::WeeklyParameters;
use super::strategic_resources::OperationalResource;
use super::strategic_resources::WeeklyResources;

// Solution fields should never be made `pub` to preserve business invariants,
// which are critical in this system.
#[derive(PartialEq, Eq, Clone)]
pub struct WeeklySolution
{
    objective_value: WeeklyObjectiveValue,
    pub(crate) strategic_scheduled_work_orders: HashMap<WorkOrderNumber, WhereIsWorkOrder<Period>>,
    pub(crate) strategic_loadings: WeeklyResources,
}

impl WeeklySolution
{
    pub fn every_work_order(&self) -> &HashMap<WorkOrderNumber, WhereIsWorkOrder<Period>>
    {
        &self.strategic_scheduled_work_orders
    }

    pub fn set_work_order_to_unschedule(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Option<WhereIsWorkOrder<Period>>
    {
        self.strategic_scheduled_work_orders
            .insert(work_order_number, WhereIsWorkOrder::NotScheduled)
    }

    pub fn set_work_order_to_strategic(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: Period,
    ) -> Option<WhereIsWorkOrder<Period>>
    {
        self.strategic_scheduled_work_orders
            .insert(work_order_number, WhereIsWorkOrder::Weekly(period))
    }

    pub fn objective_value(&self) -> &WeeklyObjectiveValue
    {
        &self.objective_value
    }
}

impl WeeklyInterface for WeeklySolution
{
    // Note: Double `Option` pattern - consider refactoring for clarity
    fn scheduled_task(
        &self,
        work_order_number: &WorkOrderNumber,
    ) -> Option<&WhereIsWorkOrder<Period>>
    {
        self.strategic_scheduled_work_orders.get(work_order_number)
    }

    fn supervisor_tasks(
        &self,
        supervisor_periods: &[Period],
    ) -> std::collections::HashMap<WorkOrderNumber, Period>
    {
        self.strategic_scheduled_work_orders
            .clone()
            .into_iter()
            .filter_map(|(won, opt_str_per)| {
                let period_option = match opt_str_per {
                    WhereIsWorkOrder::Weekly(period) => Some(period),
                    WhereIsWorkOrder::Project(period) => Some(period),
                    WhereIsWorkOrder::NotScheduled => None,
                };
                period_option
                    .and_then(|per| supervisor_periods.contains(&per).then_some((won, per)))
            })
            .collect()
    }

    fn all_scheduled_tasks(&self) -> std::collections::HashMap<WorkOrderNumber, Period>
    {
        self.strategic_scheduled_work_orders
            .clone()
            .into_iter()
            .filter_map(|(won, where_is_work_order)| {
                match where_is_work_order {
                    WhereIsWorkOrder::Weekly(period) => Some(period),
                    WhereIsWorkOrder::Project(period) => Some(period),
                    WhereIsWorkOrder::NotScheduled => None,
                }
                .map(|v| (won, v))
            })
            .collect()
    }
}

impl Debug for WeeklySolution
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            write!(
                f,
                "{}",
                format!(
                    "{:#?}\n{:<30}{}\n{:<30}{}",
                    self.objective_value,
                    "Scheduled work orders: ",
                    self.strategic_scheduled_work_orders
                        .iter()
                        .filter(|e| e.1.is_strategic_or_project())
                        .count(),
                    "Total work orders: ",
                    self.strategic_scheduled_work_orders.len()
                )
                .purple()
            )
        } else {
            write!(
                f,
                "{:#?}{:#?}{:#?}",
                self.objective_value, self.strategic_scheduled_work_orders, self.strategic_loadings
            )
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Valuable)]
pub struct WeeklyObjectiveValue
{
    pub objective_value: i64,
    pub urgency: (usize, i64),
    pub resource_penalty: (usize, i64),
    pub clustering_value: (usize, i64),
    pub percent_scheduled: (usize, Percent),
}

impl WeeklyObjectiveValue
{
    pub fn new(strategic_options: &WeeklyOptions) -> Self
    {
        Self {
            objective_value: i64::MAX,
            urgency: (strategic_options.urgency_weight, 0),
            resource_penalty: (strategic_options.resource_penalty_weight, 0),
            clustering_value: (strategic_options.clustering_weight, 0),
            percent_scheduled: (0, Percent::new(0, 100).unwrap()),
        }
    }

    pub fn aggregate_objectives(&mut self)
    {
        // Note: objective_value can be negative
        self.objective_value = self.urgency.0 as i64 * self.urgency.1
            + self.resource_penalty.0 as i64 * self.resource_penalty.1
            - self.clustering_value.0 as i64 * self.clustering_value.1;
    }
}
impl Solution for WeeklySolution
{
    type Objective = WeeklyObjectiveValue;
    type Parameters = WeeklyParameters;

    fn from_parameters(parameters: &Self::Parameters) -> Result<Self>
    {
        let strategic_loadings = parameters
            .strategic_capacity
            .0
            .iter()
            .map(|(per, res)| {
                let inner_map: HashMap<_, _> = res
                    .iter()
                    .map(|(id, or)| {
                        (
                            id.clone(),
                            OperationalResource::new(
                                id,
                                Work::from(0.0),
                                or.skill_hours.keys().cloned().collect(),
                            ),
                        )
                    })
                    .collect();

                (per.clone(), inner_map)
            })
            .collect::<HashMap<_, _>>();

        let strategic_loadings = WeeklyResources::new(strategic_loadings);

        let strategic_scheduled_work_orders = parameters
            .strategic_work_order_parameters
            .keys()
            .map(|won| (*won, WhereIsWorkOrder::NotScheduled))
            .collect();

        // TODO: Consider whether strategic options should be passed through parameters or injected as dependencies
        let strategic_objective_value = WeeklyObjectiveValue::new(&parameters.strategic_options);
        Ok(Self {
            objective_value: strategic_objective_value,
            strategic_scheduled_work_orders,
            strategic_loadings,
        })
    }

    fn update_objective(&mut self, other_objective_value: Self::Objective)
    {
        self.objective_value = other_objective_value;
    }
}

impl<Ss> SwapSolution<Ss> for WeeklySolution
where
    Ss: SystemSolutions<Weekly = WeeklySolution>,
{
    fn swap(
        id: &ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId,
        solution: SolutionState<Self>,
        system_solution: &mut Ss,
    )
    {
        system_solution.strategic_swap(id, solution);
    }
}
