pub mod assert_functions;
pub mod supervisor_interface;
pub mod supervisor_parameters;
pub mod supervisor_solution;

use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::SupervisorOptions;
use rand::rng;
use rand::seq::IndexedRandom;
use supervisor_parameters::SupervisorParameters;
use supervisor_solution::SupervisorSolution;
#[allow(unused_imports)]
use tracing::event;
#[allow(unused_imports)]
use tracing::Level;

pub struct SupervisorAlgorithm<Ss>(Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>)
where
    Ss: SystemSolutions;

impl<Ss> SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    pub fn unschedule_specific_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Result<()>
    {
        self.solution
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }
}

impl<Ss: SystemSolutions + std::fmt::Debug> std::fmt::Debug for SupervisorAlgorithm<Ss>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let supervisor_periods = &self.parameters.supervisor_periods;
        let supervisor_tasks = self
            .0
            .loaded_system_solution
            .strategic()
            .expect("The StrategicSolution should be present")
            .supervisor_tasks(supervisor_periods);

        write!(
            f,
            "{:#?}\n\
            Strategic scheduled work orders in each period:\n\
            \tFirst : {}\n\
            \tSecond: {}\n\
            \tThird : {}",
            self.0,
            supervisor_tasks
                .iter()
                .filter(|(_wo, per)| **per == supervisor_periods[0])
                .count(),
            supervisor_tasks
                .iter()
                .filter(|(_wo, per)| **per == supervisor_periods[1])
                .count(),
            supervisor_tasks
                .iter()
                .filter(|(_wo, per)| **per == supervisor_periods[2])
                .count(),
        )
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for SupervisorAlgorithm<Ss>
where
    Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>:
        AbLNSUtils<SolutionType = SupervisorSolution>,
    SupervisorSolution: Solution,
    SupervisorParameters: Parameters,
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    type Algorithm = Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>;
    type Options = SupervisorOptions;

    // I think that we can move this out
    fn make_atomic_pointer_swap(&mut self)
    {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.tactical_solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
        // NOTE
        // Every actor will have to specify how to make this work
        // on its own. There is no other way of doing it I think.
        //
        // Yes you have to make it like that. Also I can sense that
        // you will have to make the code work more efficiently with the
        // `SchedulingEnvironment` in the future.
        //
        // I do not see what other way we could make this work. The best
        // approach would possibly be
        self.arc_swap_shared_solution.rcu(|old| {
            let mut system_solutions = (**old).clone();
            // You have to invert the dependency here.
            // I cannot see how to make this function in a correct
            // manner. The best possible way here is to make the system work with
            // the required,
            SwapSolution::swap(&self.id, self.solution.clone(), &mut system_solutions);
            // <SupervisorSolution as SwapSolution>::swap(self.id, self.solution,
            // system_solutions) swap(self.id, self.solution.clone(),
            // shared_solution)
            system_solutions.supervisor_swap(&self.id, self.solution.clone());
            Arc::new(system_solutions)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    >
    {
        let assigned_woas = &self.solution.number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.solution.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let objective_value = (intermediate * 1000.0) as u64;

        // Why is there not assigned more WOs from the supervisor? There are
        // a couple of reasons, either the OperationalActors are not
        // able to insert them or the Supervisor is bad at optimizing the
        // `Delegate`s based on the otherwise good functioning of the
        // OperationalActors.
        //
        // I think that the most propable is the Operational not actually scheduling
        // that much I will include this here
        //
        // TODO [ ] 2025-07-03 Essay on the different tracing files.
        // TODO [ ] 2025-07-03 Determine how much is actually scheduled by the
        // operational actors.
        //
        if self.solution.objective_value < objective_value {
            // NOTE [ ] 2025-07-03 We should work on getting this to work correctly
            // with
            //
            let mut every_operational_assigned_or_assess_work_order_activity: HashSet<(
                WorkOrderNumber,
                ActivityNumber,
            )> = HashSet::new();
            for operational_id in self.loaded_system_solution.all_operational() {
                let total_number_of_assess_or_assign = self
                    .loaded_system_solution
                    .operational_actor_solutions(&operational_id)
                    .with_context(|| {
                        format!("The operational_actor: {operational_id} does not exist")
                    })?;

                let operational_scheduled_work_order_activities =
                    total_number_of_assess_or_assign.scheduled_activities_for_operational_actor();
                every_operational_assigned_or_assess_work_order_activity =
                    every_operational_assigned_or_assess_work_order_activity
                        .union(&operational_scheduled_work_order_activities)
                        .cloned()
                        .collect::<HashSet<_>>();
            }
            let all_work_order_activities = self
                .solution
                .operational_state_machine
                .keys()
                .map(|e| e.1)
                .collect::<HashSet<_>>();

            let share_of_schedule_work_order_activities =
                every_operational_assigned_or_assess_work_order_activity.len() as f64
                    / all_work_order_activities.len() as f64;
            event!(
                Level::INFO,
                supervisor_objective_value_better = objective_value,
                share_of_schedule_work_order_activities = share_of_schedule_work_order_activities,
            );
            Ok(ObjectiveValueType::Better(objective_value))
        } else {
            event!(
                Level::INFO,
                supervisor_objective_value_worse = objective_value
            );
            Ok(ObjectiveValueType::Worse)
        }
    }

    // Because you pulled the `schedule` out it means that you cannot create a
    // supervisor specific error code. Is this an issue? I do not think that it
    // is actually. I simply means that you have to think carefully about how
    // you structure your error messages.
    //
    // Good you are ready to move on now.
    // ISSUE Start here [ ]
    // You have to fix the initialization.
    // TODO [ ]
    // FIX the supervisor initialization.
    fn schedule(&mut self) -> Result<()>
    {
        // What is the criteria for handling this in practice?

        // FIX [ ]
        // We should first make sure that the `Supervisor` is actually working with the
        // correct state.
        //
        // TODO [ ]
        // We want to know how the `StrategicSolution` looks like when an error occurs
        //
        // in the
        // This will always fail as you do not have the correct... You want the
        // supervisor to see these, but you do not want to have them in the
        // solution. I am not sure what the best approach is here.
        //
        // Where should the discrepancy be handled? I think that the best place is in
        // the I think that the Supervisor should be able to see what is
        // suggested to him and The issue is where to put the information. I am
        // not really sure what the best place is to do this! The question is if
        // we want to incorporate this into the state of the supervisor... I
        // actually do not think that is something that we want. A key insight
        // of the architecture is that the state of the other algorithms are
        // always available. And that we should use this as much as possible.
        //
        // How to tackle this problem then? Okay now we need to make sure that
        // the code runs correctly with
        // TODO [ ]
        // Debug the SupervisorActor.
        ensure!(
            self.loaded_system_solution
                .strategic()
                .unwrap()
                .supervisor_tasks(&self.parameters.supervisor_periods)
                .len()
                >= self
                    .solution
                    .get_work_order_activities()
                    .iter()
                    .map(|e| e.0)
                    .collect::<HashSet<_>>()
                    .len(),
            "{} Strategic workorders in supervisor interval\n\
            {} Supervisor workorders in supervisor interval\n\
            {} activities in the Supervisor Solution\n\
            {} `WorkOrder`s in the Supervisor parameters\n\
            {} `Activity`s in the SupervisorParameters\n\
            Location: {}",
            self.loaded_system_solution
                .strategic()
                .unwrap()
                .supervisor_tasks(&self.parameters.supervisor_periods)
                .len(),
            self.solution
                .get_work_order_activities()
                .iter()
                .map(|e| e.0)
                .collect::<HashSet<_>>()
                .len(),
            self.solution.get_work_order_activities().len(),
            self.parameters.supervisor_work_orders.len(),
            self.parameters
                .supervisor_work_orders
                .iter()
                .fold(0, |acc, count_activities| {
                    acc + count_activities.1.len()
                }),
            Location::caller(),
        );

        for work_order_activity in &self.solution.get_work_order_activities() {
            let number = self
                .parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number;

            let mut operational_status_by_work_order_activity =
                self.solution.operational_status_by_work_order_activity(
                    work_order_activity,
                    &self.loaded_system_solution,
                )?;

            operational_status_by_work_order_activity
                .retain(|(_, _, mar_fit)| matches!(mar_fit, MarginalFitness::Scheduled(_)));

            operational_status_by_work_order_activity.sort_by_key(|(_agent_id, _, mar_fit)| {
                match mar_fit {
                    MarginalFitness::Scheduled(auxillary_operational_objective) => {
                        *auxillary_operational_objective
                    }
                    MarginalFitness::None => panic!(),
                }
            });

            if !operational_status_by_work_order_activity.is_empty() {};

            let number_of_assigned = operational_status_by_work_order_activity
                .iter()
                .filter(|(_, delegate, _)| *delegate == Delegate::Assign)
                .count() as u64;

            let mut remaining_to_assign = number - number_of_assigned;

            event!(Level::DEBUG, remaining_to_assign = ?remaining_to_assign);
            for (agent_id, delegate_status, _marginal_fitness) in
                operational_status_by_work_order_activity.clone()
            {
                if delegate_status != Delegate::Assess {
                    continue;
                }

                let solution =
                    self.solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails");

                if remaining_to_assign >= 1 {
                    remaining_to_assign -= 1;
                    solution.state_change_to_assign();
                } else {
                    if delegate_status == Delegate::Assign {
                        continue;
                    }
                    solution.state_change_to_unassign();
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self) -> Result<()>
    {
        let mut rng = rng();
        let work_order_numbers = self.solution.get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(
                &mut rng,
                self.parameters.options.number_of_unassigned_work_orders,
            )
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!("Could not unschedule work_order_number: {work_order_number:?}")
                })?;
        }
        Ok(())
        // self.algorithm.operational_state.
        // assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&
        // old_state).unwrap();
    }

    fn incorporate_system_solution(&mut self) -> Result<bool>
    {
        // List current activities in the `SupervisorAgent`
        let current_activities = self
            .solution
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa.0)
            .collect::<HashSet<WorkOrderNumber>>();

        // Filter for Strategic scheduled work orders that are inside of the
        // `SupervisorAlgorithm.parameters.strategic_periods`. This can be made
        // cleaner! Much cleaner,
        let strategic_activities_in_supervisor_period = self
            .loaded_system_solution
            .strategic()?
            .supervisor_tasks(&self.parameters.supervisor_periods);

        // Select only those that are not part of the `SupervisorAgent` already
        let incoming_activities = strategic_activities_in_supervisor_period
            .iter()
            .filter(|(won, _)| !current_activities.contains(won));

        // Insert all the incoming activities as Delegate::default() for each
        // `OperationalAgent` that has the required skill, `enum Resources`
        // QUESTION
        // Why does this happen here? I do not really know why and that is an
        // issue. You should find out now.
        //
        // TODO [ ]
        // determine exactly how to fix this.
        //
        let work_order_parameters = self.parameters.supervisor_work_orders.clone();
        let all_operational_actors = self.loaded_system_solution.all_operational().clone();

        // Infeasible [`WorkOrder`]s

        let mut non_incorporated_work_orders: HashSet<WorkOrderNumber> = HashSet::new();
        for (work_order_number, _) in incoming_activities {
            let activity_number = work_order_parameters
                .get(work_order_number)
                .context("Missing WorkOrder Parameter in Supervisor")?
                .keys()
                .cloned();

            // IMPORTANT. The supervisor and strategic state does not necessarily
            // have to be the same... As there are cases where the strategic
            // work order cannot be accepted into the supervisor state. This means
            // that it is crucial. That the work is added no matter what.
            for activity_number in activity_number {
                for operational_id in &all_operational_actors {
                    let supervisor_parameter = self
                        .parameters
                        .supervisor_work_orders
                        .get(work_order_number)
                        .context("Missing WorkOrder Parameter in Supervisor")?
                        .get(&activity_number)
                        .context("Missing Activity Parameter in Supervisor")?;

                    let supervisor_parameter_resource = &supervisor_parameter.resource;

                    if supervisor_parameter.work == Work::from(0.0) {
                        non_incorporated_work_orders.insert(*work_order_number);
                        continue;
                    };

                    if operational_id.1.contains(supervisor_parameter_resource) {
                        let work_order_activity = (*work_order_number, activity_number);
                        let operational_state = ((*operational_id).clone(), work_order_activity);

                        self.solution
                            .operational_state_machine
                            .insert(operational_state, Delegate::default());
                    } else {
                        non_incorporated_work_orders.insert(*work_order_number);
                    }
                }
            }
        }

        let strategic_activities = strategic_activities_in_supervisor_period
            .iter()
            .map(|e| e.0)
            .cloned()
            .collect::<HashSet<_>>();

        self.solution
            .operational_state_machine
            .retain(|id_woa, _| strategic_activities.contains(&id_woa.1 .0));

        let supervisor_work_orders = self
            .solution
            .operational_state_machine
            .iter()
            .map(|e| e.0 .1 .0)
            .collect::<HashSet<_>>();
        // Okay now we want to run this based on the state of the `Supervisor`
        //
        // If we get an `Error` here we should expand the code to incorporate the
        // missing link here. That means that you should focus on understanding
        // the root cause if this new change errors.
        //
        // What does it mean that this is not in the correct place?
        // There are more
        // So the sets should be part of the of the sets that is the smallest.
        //
        // That means that we should take the `union` between the two
        // difference.
        // You should find a more trivial way of storing all.
        // Good. You logic works. But what should be done about the
        // difference?

        // So there are two different issues here.
        // * Either the work is zero in which case the activities will be excluded
        // * There is no underlying resource available to fix the issue.
        // I think that we should clearly understand what should be done here.
        //
        // I think that there is a fundamental issue here where
        // the [`SchedulingEnvironment`] does not have any way
        // of incorporating the state of the supervisor in the
        // [`WorkOrder`]s.
        //
        // ESSAY [ ]
        // You should think about how you incorporate the state
        // here. It is unreliable to rely on the solution as the
        // solutions can change easily. I am not sure where to
        // put this.
        // What should happen to the work orders that does
        // not fit well into the model.
        //
        // The fundamental issue is how to give the work orders to. I think that
        // they should stay in the [`SupervisorActor`] so that he can work on the
        // with them and send them where they need to be, and maybe even manually
        // assign the activity to a technician.
        //
        // I believe that including it into the Supervisor is the best approach for now.
        ensure!(
            strategic_activities
                == supervisor_work_orders
                    .union(&non_incorporated_work_orders)
                    .cloned()
                    .collect(),
            "Strategic activities: {:#?}\n\
             Supervisor solution: {:#?}\n\
             difference between strategic / supervisor: {:#?}\n\
             difference between supervisor / strategic: {:#?}",
            strategic_activities,
            supervisor_work_orders.union(&non_incorporated_work_orders),
            strategic_activities.difference(
                &supervisor_work_orders
                    .union(&non_incorporated_work_orders)
                    .cloned()
                    .collect::<HashSet<_>>()
            ),
            supervisor_work_orders
                .union(&non_incorporated_work_orders)
                .cloned()
                .collect::<HashSet<_>>()
                .difference(&strategic_activities),
        );

        // After all the state has been in corporated, an [`ArcSwap`] must be
        // performed.
        // NOTE 2025-06-28
        Ok(true)
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm
    {
        &mut self.0
    }
}

fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool
{
    assigned_woas
        .iter()
        .all(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
}
impl<Ss> Deref for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target = Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}
impl<Ss> From<Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>>
    for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(value: Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>) -> Self
    {
        SupervisorAlgorithm(value)
    }
}
