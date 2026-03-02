use std::fmt::Debug;
use std::panic::Location;
use std::sync::Arc;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use ordinator_orchestrator_actor_traits::Options;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;

use crate::traits::AbLNSUtils;

#[derive(Debug)]
pub struct Algorithm<S, P, I, O, Ss>
where
    S: Solution,
    P: Parameters,
    O: Options,
    Ss: SystemSolutions,
{
    pub id: ActorCompositeId,
    pub solution_intermediate: I,
    pub solution: SolutionState<S>,
    pub parameters: P,
    pub options: O,
    pub arc_swap_shared_solution: Arc<ArcSwap<Ss>>,
    pub loaded_system_solution: Guard<Arc<Ss>>,
}

/// Builder for constructing an [`Algorithm`] with required parameters.
pub struct AlgorithmBuilder<S, P, I, O, Ss>
where
    S: Solution,
    P: Parameters,
    O: Options,
    Ss: SystemSolutions,
{
    id: Option<ActorCompositeId>,
    solution_intermediate: I,
    solution: Option<SolutionState<S>>,
    parameters: Option<P>,
    options: Option<O>,
    arc_swap_shared_solution: Option<Arc<ArcSwap<Ss>>>,
    loaded_shared_solution: Option<Guard<Arc<Ss>>>,
}

impl<S, P, I, O, Ss> Algorithm<S, P, I, O, Ss>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
    O: Options,
    Ss: SystemSolutions,
{
    pub fn builder() -> AlgorithmBuilder<S, P, I, O, Ss>
    {
        AlgorithmBuilder {
            id: None,
            solution_intermediate: I::default(),
            solution: None,
            parameters: None,
            options: None,
            arc_swap_shared_solution: None,
            loaded_shared_solution: None,
        }
    }
}

impl<S, P, I, O, Ss> AbLNSUtils for Algorithm<S, P, I, O, Ss>
where
    I: Default,
    S: Solution + Debug + Clone,
    P: Parameters,
    O: Options,
    Ss: SystemSolutions,
{
    type SolutionType = S;

    fn clone_algorithm_solution(&self) -> S
    {
        self.solution.inner().clone()
    }

    fn load_shared_solution(&mut self)
    {
        self.loaded_system_solution = self.arc_swap_shared_solution.load();
    }

    fn swap_to_old_solution(&mut self, solution: S)
    {
        // Revert the solution and update internal counters
        self.solution.revert_to_old_solution(solution);
    }

    fn update_objective(&mut self, objective_value: <Self::SolutionType as Solution>::Objective)
    {
        self.solution.update_objective(objective_value);
    }
}

// Generic parameter `Alg` allows custom type conversions from Algorithm.
impl<S, P, I, O, Ss> AlgorithmBuilder<S, P, I, O, Ss>
where
    S: SwapSolution<Ss> + Solution<Parameters = P> + Clone,
    P: Parameters<Options = O>,
    I: Default,
    O: Options,
    Ss: SystemSolutions,
{
    pub fn build<Alg>(self) -> Result<Alg>
    where
        Algorithm<S, P, I, O, Ss>: Into<Alg>,
    {
        let algorithm_inner = Algorithm {
            id: self.id.unwrap(),
            solution_intermediate: self.solution_intermediate,
            solution: self.solution.unwrap(),
            parameters: self.parameters.unwrap(),
            options: self.options.expect("Call `options()` build method first"),
            arc_swap_shared_solution: self.arc_swap_shared_solution.unwrap(),
            loaded_system_solution: self.loaded_shared_solution.unwrap(),
        };

        Ok(algorithm_inner.into())
    }

    pub fn id(mut self, id: ActorCompositeId) -> Self
    {
        self.id = Some(id);
        self
    }

    pub fn options(mut self, options: O) -> Self
    {
        self.options = Some(options);
        self
    }

    pub fn parameters_and_solution(
        mut self,
        scheduling_environment: &MutexGuard<SchedulingHypergraph>,
        options: O,
    ) -> Result<Self>
    {
        let parameters = P::from_scheduling_hypergraph(
            self.id.as_ref().expect("Call `id()` build method first"),
            scheduling_environment,
            &options,
        )?;

        // S is the concrete type, Solution is the trait
        self.solution = Some(SolutionState::new(
            S::from_parameters(&parameters).with_context(|| {
                format!(
                    "Could not build solution from parameters\nLocation: {}",
                    Location::caller()
                )
            })?,
        ));

        self.parameters = Some(parameters);
        self.options = Some(options);

        Ok(self)
    }

    pub fn system_solution_arc_swap(
        mut self,
        system_solution_arc_swap: Arc<ArcSwap<Ss>>,
    ) -> Result<Self>
    where
        Ss: SystemSolutions,
    {
        self.arc_swap_shared_solution = Some(system_solution_arc_swap);
        // Individual solutions specify how to swap within the system solution
        self.arc_swap_shared_solution.as_ref().unwrap().rcu(|old| {
            let mut system_solution = (**old).clone();
            SwapSolution::swap(
                self.id.as_ref().unwrap(),
                self.solution.as_ref().unwrap().clone(),
                &mut system_solution,
            );
            Arc::new(system_solution)
        });

        self.loaded_shared_solution = Some(
            self.arc_swap_shared_solution
                .as_ref()
                .expect("Set the `arc_swap` field first")
                .load(),
        );

        Ok(self)
    }
}

pub enum LoadOperation
{
    Add,
    Sub,
}
