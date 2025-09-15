use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use arc_swap::Guard;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;

use super::supervisor_parameters::SupervisorParameters;

pub type SupervisorObjectiveValue = u64;

#[derive(PartialEq, Eq, Default, Clone)]
pub struct SupervisorSolution
{
    pub(crate) objective_value: SupervisorObjectiveValue,
    pub(crate) operational_state_machine: HashMap<(ActorCompositeId, WorkOrderActivity), Delegate>,
}

impl std::fmt::Debug for SupervisorSolution
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("SupervisorSolution")
            .field("objective_value", &self.objective_value)
            .field(
                "operational_state_machine_technicians",
                &self
                    .operational_state_machine
                    .iter()
                    .map(|e| e.0.0.clone())
                    .collect::<HashSet<_>>()
                    .len(),
            )
            .field(
                "operational_state_machine_work_order_activities",
                &self
                    .operational_state_machine
                    .iter()
                    .map(|e| e.0.1)
                    .collect::<HashSet<_>>()
                    .len(),
            )
            .finish()
    }
}

impl SupervisorSolution
{
    pub fn new_from_parts(
        operational_state_machine: HashMap<(ActorCompositeId, WorkOrderActivity), Delegate>,
    ) -> Self
    {
        Self {
            objective_value: SupervisorObjectiveValue::default(),
            operational_state_machine,
        }
    }
}

// impl std::fmt::Debug for SupervisorSolution
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
//     {
//         if f.alternate() {
//             write!(
//                 f,
//                 "SupervisorSolution \
//                 {{\n\
//                 \tobjective_value: {:#?}\n\
//                 \toperational_state_machine: {}\n\
//                 }}",
//                 self.objective_value,
//                 self.operational_state_machine.len(),
//             )
//         } else {
//         }
//     }
// }

impl Solution for SupervisorSolution
{
    type Objective = SupervisorObjectiveValue;
    type Parameters = SupervisorParameters;

    // Here the solution should not be created based on the
    // state of the parameters but on the state in the strategic
    // actor. That is what you want to do here.
    fn from_parameters(_parameters: &Self::Parameters) -> Result<Self>
    {
        // The SupervisorParameters should have knowledge of the agents.
        // Does that mean that you simply have to instantiate a single
        // empty `HashMap`.
        // That means that you should comment this out and then work on the
        // This should maybe be moved to the `incorporate_system_solution`

        let operational_state_machine = HashMap::new();
        // ISSUE #000
        // let operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate> =
        // parameters     .supervisor_work_orders
        //     .iter()
        //     .flat_map(|(won, inner)| {
        //         inner.iter().flat_map(|(acn, sp)| {
        //             // So here is the fundamental issue in the code. We have
        //             // a parameters that is initialized first and synchronously. This
        //             // means that we should work on the best way to make the code.
        //             //
        //             // We should make sure that this works in the best way possible.
        //             //
        //             // The flow is `SchedulingEnvironment` -> `Parameters` ->
        // `Solution`             //
        //             // This flow means that the if the `Solution` is inconsistent
        // with the             // `Parameters` that is okay, but not the other
        // way around.             parameters
        //                 .operational_ids
        //                 .iter()
        //                 .filter(|e| e.1.contains(&sp.resource))
        //                 .map(|e| ((e.clone(), (*won, *acn)), Delegate::Assess))
        //         })
        //     })
        //     .collect();

        let objective_value = Self::Objective::default();

        Ok(Self {
            objective_value,
            operational_state_machine,
        })
    }

    fn update_objective(&mut self, other_objective_value: Self::Objective)
    {
        self.objective_value = other_objective_value;
    }
}

impl<Ss> SwapSolution<Ss> for SupervisorSolution
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    fn swap(id: &ActorCompositeId, solution: SolutionState<Self>, system_solution: &mut Ss)
    {
        system_solution.supervisor_swap(id, solution);
    }
}
/// The SupervisorSolution is a state machine that keeps track of all the
/// states of the operational agents. It is a solution representation of
/// a **iterative combinatorial auction algorithms**.
///
/// We should be careful about how we implement this system.
impl SupervisorSolution
{
    pub fn all_technicians(&self) -> BTreeSet<ActorCompositeId>
    {
        self.operational_state_machine
            .keys()
            .map(|e| e.0.clone())
            .collect()
    }

    pub fn assigned_activities(&self) -> BTreeMap<ActorCompositeId, WorkOrderActivity>
    {
        self.operational_state_machine
            .iter()
            .filter(|e| e.1.is_assign())
            .map(|(e, _)| (e.0.clone(), e.1))
            .collect()
    }

    // This is implemented incorrectly. You should make the code work correctly with
    // the. This means that each Id can only have a single WorkOrderActivity.
    pub fn assess_and_assign_activities(&self) -> Vec<(ActorCompositeId, WorkOrderActivity)>
    {
        self.operational_state_machine
            .iter()
            .filter(|e| e.1.is_assign() || e.1.is_assess())
            .map(|(e, _)| (e.0.clone(), e.1))
            .collect()
    }

    pub fn turn_work_order_into_delegate_assess(&mut self, work_order_number: WorkOrderNumber)
    {
        self.operational_state_machine
            .iter_mut()
            .filter(|(key, _)| key.1.0 == work_order_number)
            .for_each(|(_, delegate)| *delegate = Delegate::Assess)
    }

    pub fn count_unique_woa(&self) -> usize
    {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .len()
    }

    pub fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity>
    {
        self.operational_state_machine
            .iter()
            .filter(|(_, val)| val.is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    pub fn operational_status_by_work_order_activity<Ss>(
        &self,
        work_order_activity: &WorkOrderActivity,
        loaded_shared_solution: &Guard<Arc<Ss>>,
    ) -> Result<Vec<(ActorCompositeId, Delegate, MarginalFitness)>>
    where
        Ss: SystemSolutions,
    {
        let mut out = Vec::new();
        for (id_woa, delegate) in self
            .operational_state_machine
            .iter()
            // We only take the work_orders_that are actually there.
            .filter(|id_and_woa| id_and_woa.0.1 == *work_order_activity)
            .filter(|id_and_woa| id_and_woa.1 == &Delegate::Assess)
        {
            // We only consider the delagates if the operational actor is actually running.
            if let Ok(operational_solution) =
                loaded_shared_solution.operational_actor_solutions(&id_woa.0)
            {
                // OperationalActor might not have had enough time to incorporate the
                // state, in that case `None` is returned
                let op = operational_solution
                    .marginal_fitness_for_operational_actor(work_order_activity);

                if let Some(fitness) = op
                    && matches!(fitness, MarginalFitness::Scheduled(_))
                {
                    out.push((id_woa.0.clone(), delegate.clone(), fitness.clone()));
                }
            };
        }

        // There can be no duplicates
        out.sort_by_key(|(_agent_id, _, mar_fit)| match mar_fit {
            MarginalFitness::Scheduled(auxillary_operational_objective) => {
                *auxillary_operational_objective
            }
            MarginalFitness::None => unreachable!(),
        });
        Ok(out)
    }

    #[allow(dead_code)]
    pub(crate) fn get_iter(
        &'_ self,
    ) -> std::collections::hash_map::Iter<'_, (ActorCompositeId, WorkOrderActivity), Delegate>
    {
        self.operational_state_machine.iter()
    }

    pub(crate) fn get_assigned_and_unassigned_work_orders(&self) -> Vec<WorkOrderNumber>
    {
        self.operational_state_machine
            .iter()
            .filter(|(_, delegate)| {
                **delegate == Delegate::Assign || **delegate == Delegate::Unassign
            })
            .map(|(id_woa, _)| id_woa.1.0)
            .collect()
    }

    pub(crate) fn get_work_order_activities(&self) -> HashSet<WorkOrderActivity>
    {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect()
    }
}
