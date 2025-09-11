pub mod assert_functions;
pub mod supervisor_interface;
pub mod supervisor_parameters;
pub mod supervisor_solution;

use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::OperationalInterface;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::SupervisorOptions;
use rand::rng;
use rand::seq::IndexedRandom;
use supervisor_parameters::SupervisorParameters;
use supervisor_solution::SupervisorSolution;
#[allow(unused_imports)]
use tracing::Level;
#[allow(unused_imports)]
use tracing::event;

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
        ObjectiveValueType<<<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective>,
    >
    {
        let assigned_woas = &self.solution.number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.solution.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let new_objective_value = (intermediate * 1000.0) as u64;

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
        if self.solution.objective_value < new_objective_value {
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
                target: "research",
                Level::INFO,
                supervisor_objective_accepted = new_objective_value,
                share_of_schedule_work_order_activities = share_of_schedule_work_order_activities,
                reason = "optimization loop found a better solution",
            );
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            event!(
                target: "research",
                Level::TRACE,
                supervisor_objective_rejected = new_objective_value
            );
            Ok(ObjectiveValueType::Worse(new_objective_value))
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
            let number_of_people = self
                .parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number_of_people;

            let operational_status_by_work_order_activity =
                self.solution.operational_status_by_work_order_activity(
                    work_order_activity,
                    &self.loaded_system_solution,
                )?;

            // Ahh you filter them out here! So that there will be none left
            ensure!(
                operational_status_by_work_order_activity
                    .iter()
                    .all(|e| matches!(e.1, Delegate::Assess))
            );

            // This should be based on the current solution instead of the derived data!
            // Crucial insight!
            let number_of_assigned = self
                .solution
                .operational_state_machine
                .iter()
                .filter(|(b, delegate)| {
                    work_order_activity == &b.1 && delegate == &&Delegate::Assign
                })
                .count() as u64;

            let mut remaining_to_assign = number_of_people
                .checked_sub(number_of_assigned)
                .with_context(|| format!("Failed to subtract `number_of_people`: {number_of_people}\nfrom the `number_of_assigned`: {number_of_assigned}\nto be assigned to `work_order_activity`: {work_order_activity:?}"))?;

            ensure!(
                remaining_to_assign <= 1,
                "Failed to subtract `number_of_people`: {number_of_people}\nfrom the `number_of_assigned`: {number_of_assigned}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
                Location::caller()
            );

            for (actor_id, mut temporary_technician_delegate, _marginal_fitness) in
                operational_status_by_work_order_activity.clone()
            {
                ensure!(matches!(temporary_technician_delegate, Delegate::Assess));

                let value = self
                    .solution
                    .operational_state_machine
                    .iter()
                    .filter(|e| e.0.1 == *work_order_activity && e.1.is_assign())
                    .count();
                ensure!(
                    value as u64 <= number_of_people,
                    "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people}\n{}\nto be assigned to `work_order_activity`: {work_order_activity:?}",
                    Location::caller()
                );
                let technician_delegate =
                    self.solution
                        .operational_state_machine
                        .get_mut(&(actor_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails");

                event!(target: "debug",Level::DEBUG, delegate_status = ?temporary_technician_delegate, solution_delegate = ?technician_delegate, work_order_activity = ?work_order_activity);
                if remaining_to_assign >= 1 {
                    remaining_to_assign -= 1;
                    event!(target: "debug", Level::DEBUG, work_order_activity = ?work_order_activity, technician = ?actor_id, "assigning `work_order_activity` to technician");
                    event!(target: "developer", Level::DEBUG, delegate_status = ?temporary_technician_delegate, solution_delegate = ?technician_delegate, work_order_activity = ?work_order_activity);

                    // Solution comes from the `Supervisor`.

                    ensure!(matches!(temporary_technician_delegate, Delegate::Assess));
                    technician_delegate
                        .state_change_to_assign()
                        .with_context(|| format!("{}", Location::caller()))?;
                    let value = self
                        .solution
                        .operational_state_machine
                        .iter()
                        .filter(|e| e.0.1 == *work_order_activity && e.1.is_assign())
                        .count();
                    ensure!(
                        value as u64 <= number_of_people,
                        "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
                        Location::caller()
                    )
                } else {
                    // if delegate_status == Delegate::Assign {
                    //     continue;
                    // }
                    event!(target: "debug", Level::DEBUG, work_order_activity = ?work_order_activity, technician = ?actor_id, "unassigning `work_order_activity` to technician");
                    event!(target: "developer", Level::DEBUG, delegate_status = ?temporary_technician_delegate, solution_delegate = ?technician_delegate, work_order_activity = ?work_order_activity);
                    technician_delegate
                        .state_change_to_unassign()
                        .with_context(|| format!("{}", Location::caller()))?;
                    temporary_technician_delegate
                        .state_change_to_unassign()
                        .with_context(|| format!("{}", Location::caller()))?;
                }
                let value = self
                    .solution
                    .operational_state_machine
                    .iter()
                    .filter(|e| e.0.1 == *work_order_activity && e.1.is_assign())
                    .count();
                ensure!(
                    value as u64 <= number_of_people,
                    "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
                    Location::caller()
                )
            }

            let value = self
                .solution
                .operational_state_machine
                .iter()
                .filter(|e| e.0.1 == *work_order_activity && e.1.is_assign())
                .count();
            ensure!(
                value as u64 <= number_of_people,
                "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
                Location::caller()
            )
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
            self.solution
                .operational_state_machine
                .iter_mut()
                .filter(|(key, _)| key.1.0 == *work_order_number)
                .for_each(|(_, delegate)| *delegate = Delegate::Assess);
        }

        Ok(())
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

                    if supervisor_parameter.work_remaining == Work::from(0.0) {
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
            .retain(|id_woa, _| strategic_activities.contains(&id_woa.1.0));

        let supervisor_work_orders = self
            .solution
            .operational_state_machine
            .iter()
            .map(|e| e.0.1.0)
            .collect::<HashSet<_>>();
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

    fn force_schedule(&mut self) -> Result<()>
    {
        todo!()
    }

    fn throttling(&self, throttling: &ordinator_configuration::throttling::Throttling) -> u64
    {
        throttling.supervisor_throttling
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
