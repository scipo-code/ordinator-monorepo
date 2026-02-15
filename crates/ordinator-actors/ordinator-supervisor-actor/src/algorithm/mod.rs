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
use chrono::Local;
use chrono::Timelike;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::Percent;
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

    fn make_atomic_pointer_swap(&mut self)
    {
        // TODO: Consider extracting this method.
        // Performance optimization opportunities:
        // - Use Copy-on-Write pattern to avoid cloning entire SharedSolution
        // - Clone only the fields that have changed, reuse others
        // Note: Each actor must implement its own version of this swap logic
        self.arc_swap_shared_solution.rcu(|old| {
            let mut system_solutions = (**old).clone();
            SwapSolution::swap(&self.id, self.solution.clone(), &mut system_solutions);
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

        let new_objective_value = Percent::new(assigned_woas.len() as u64, all_woas.len() as u64)
            .with_context(|| format!("{}", Location::caller()))?;

        // TODO: Determine how much is actually scheduled by operational actors to improve optimization
        if self.solution.objective_value < new_objective_value {
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            Ok(ObjectiveValueType::Worse(new_objective_value))
        }
    }

    // TODO: Fix supervisor initialization logic
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
            let number_of_people_for_operation = self
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

            ensure!(
                operational_status_by_work_order_activity
                    .iter()
                    .all(|e| matches!(e.1, Delegate::Assess))
            );

            // Count assigned delegates from the current solution
            let number_of_assigned = self
                .solution
                .operational_state_machine
                .iter()
                .filter(|(b, delegate)| {
                    work_order_activity == &b.1 && delegate == &&Delegate::Assign
                })
                .count() as u64;

            let mut remaining_to_assign = number_of_people_for_operation
                .checked_sub(number_of_assigned)
                .with_context(|| format!("Failed to subtract `number_of_people_for_operation`: {number_of_people_for_operation}\nfrom the `number_of_assigned`: {number_of_assigned}\nto be assigned to `work_order_activity`: {work_order_activity:?}"))?;

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
                    value as u64 <= number_of_people_for_operation,
                    "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people_for_operation}\n{}\nto be assigned to `work_order_activity`: {work_order_activity:?}",
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
                        value as u64 <= number_of_people_for_operation,
                        "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people_for_operation}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
                        Location::caller()
                    )
                } else {
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
                    value as u64 <= number_of_people_for_operation,
                    "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people_for_operation}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
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
                value as u64 <= number_of_people_for_operation,
                "number of Delegate::Assign: {value}\nnumber_of_people: {number_of_people_for_operation}\nto be assigned to `work_order_activity`: {work_order_activity:?}\n{}",
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
        // Collect current work order activities in the supervisor solution
        let current_activities = self
            .solution
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa.0)
            .collect::<HashSet<WorkOrderNumber>>();

        // Fetch strategic work orders scheduled within the supervisor period
        let strategic_activities_in_supervisor_period = self
            .loaded_system_solution
            .strategic()?
            .supervisor_tasks(&self.parameters.supervisor_periods);

        // Filter to only new activities not already in the supervisor solution
        let incoming_activities = strategic_activities_in_supervisor_period
            .iter()
            .filter(|(won, _)| !current_activities.contains(won));

        // Assign incoming activities to operational agents with required resources
        let work_order_parameters = self.parameters.supervisor_work_orders.clone();
        let all_operational_actors = self.loaded_system_solution.all_operational().clone();

        // Track work orders that cannot be incorporated due to infeasibility or missing resources
        let mut non_incorporated_work_orders: HashSet<WorkOrderNumber> = HashSet::new();
        for (work_order_number, _) in incoming_activities {
            let activity_number = work_order_parameters
                .get(work_order_number)
                .context("Missing WorkOrder Parameter in Supervisor")?
                .keys()
                .cloned();

            // Note: Strategic and supervisor states may differ; work is added if feasible
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
