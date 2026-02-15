use std::collections::HashSet;

use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;

use super::operational_solution::OperationalSolution;

impl OperationalInterface for OperationalSolution
{
    fn marginal_fitness_for_operational_actor<'a>(
        &'a self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<&'a MarginalFitness>
    {
        self.scheduled_work_order_activities
            .iter()
            .find(|woa_ass| woa_ass.0 == *work_order_activity)
            .map(|woa_ass| &woa_ass.1.marginal_fitness)
    }

    fn scheduled_activities_for_operational_actor(
        &self,
    ) -> std::collections::HashSet<WorkOrderActivity>
    {
        self.scheduled_work_order_activities
            .iter()
            .map(|e| e.0)
            .collect::<HashSet<_>>()
    }
}
