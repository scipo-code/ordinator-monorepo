use std::fmt::Debug;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_configuration::throttling::Throttling;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_scheduling_environment::SchedulingEnvironment;
use serde::Serialize;
use tracing::event;
use tracing::Level;
use valuable::Valuable;

pub type ActorLinkToSchedulingEnvironment<'a> = MutexGuard<'a, SchedulingEnvironment>;

pub trait ActorBasedLargeNeighborhoodSearch
{
    type Algorithm: AbLNSUtils;
    type Options;

    // TODO: Avoid locking the scheduling environment on every iteration. Weights are cached
    // on the workorder, and configuration changes should be reflected dynamically. See ISSUE #129.
    fn run_lns_iteration(&mut self) -> Result<()>
        where
            <<<Self as ActorBasedLargeNeighborhoodSearch>::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective: Valuable
    {
        // TODO: Options should be part of the Algorithm or Actor with dependency injection
        self.update_based_on_system_solution().with_context(|| {
            format!(
                "Could not update the Algorithm state based on SystemSolution\nLocation: {}:{}",
                file!(),
                line!()
            )
        })?;

        let current_solution = self.algorithm_util_methods().clone_algorithm_solution();

        self.unschedule()
            .with_context(|| format!("{current_solution:#?}"))?;

        self.schedule()
            .with_context(|| format!("Could not schedule\n{current_solution:#?}"))?;

        let objective_value_type = self.calculate_objective_value().with_context(|| {
            format!(
                "Could not calculate the objective value\nLocation: {}:{}",
                file!(),
                line!()
            )
        })?;

        match objective_value_type {
            ObjectiveValueType::Better(objective_value) => {
                event!(target: "research", Level::INFO, objective_value = objective_value.as_value(), reason = "optimization loop found a better solution");
                self.algorithm_util_methods()
                    .update_objective(objective_value);
                self.make_atomic_pointer_swap();
            }
            ObjectiveValueType::Worse(objective_value) => {
                event!(target: "research", Level::DEBUG, objective_value = objective_value.as_value(), reason = "optimization loop found a worse solution");
                self
                    .algorithm_util_methods()
                    .swap_to_old_solution(current_solution);
            }
            ObjectiveValueType::Force(_) => todo!(),
        }
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm;

    fn make_atomic_pointer_swap(&mut self);

    // TODO: State link should handle scheduling environment locks to avoid conflicts
    // when options are updated dynamically
    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<<<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective>,
    >;

    fn schedule(&mut self) -> Result<()>;

    fn force_schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self) -> Result<()>;

    /// This method is for updating the algorithm based on external inputs and
    /// the shared solution. That means that this method has to look at relevant
    /// state in the others `Agent`s and incorporate that and handled changes in
    /// parameters coming from external inputs.
    fn update_based_on_system_solution(&mut self) -> Result<()>
    {
        self.algorithm_util_methods().load_shared_solution();

        let state_change = self.incorporate_system_solution()?;

        if state_change {
            // TODO: Refactor objective calculation to avoid redundant scheduling logic
            let objective = self.calculate_objective_value().with_context(|| format!("Could not calculate the objective value after a incorporating state from the system solution\nLocation: {}:{}", file!(), line!()))?;
            match objective {
                ObjectiveValueType::Better(objective) => {
                    event!(target: "research", Level::INFO, objective_value = objective.as_value(), reason = "state incorporation from system solution");
                    self.algorithm_util_methods().update_objective(objective);
                }
                ObjectiveValueType::Worse(objective) => {
                    event!(target: "research", Level::INFO, objective_value = objective.as_value(), reason = "state incorporation from system solution");
                    self.algorithm_util_methods().update_objective(objective);
                }
                ObjectiveValueType::Force(objective) => {
                    event!(target: "research", Level::INFO, objective_value = objective.as_value(), reason = "state incorporation from system solution");
                    self.algorithm_util_methods().update_objective(objective);
                }
            }
            self.make_atomic_pointer_swap();
        }

        Ok(())
    }

    fn incorporate_system_solution(&mut self) -> Result<bool>;
    fn throttling(&self, throttling: &Throttling) -> u64;
}

pub trait AbLNSUtils
{
    type SolutionType: Solution + Debug + Clone;

    fn clone_algorithm_solution(&self) -> Self::SolutionType;

    fn load_shared_solution(&mut self);

    // Updates objective value for the solution
    fn update_objective(&mut self, objective_value: <Self::SolutionType as Solution>::Objective);

    fn swap_to_old_solution(&mut self, solution: Self::SolutionType);
}

#[derive(Debug)]
pub enum ObjectiveValueType<O>
{
    Better(O),
    Worse(O),
    Force(O),
}

pub trait ObjectiveValue: Serialize {}
