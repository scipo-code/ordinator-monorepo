mod assert_functions;
pub mod tactical_interface;
pub mod tactical_parameters;
pub mod tactical_resources;
pub mod tactical_solution;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use chrono::NaiveDate;
use chrono::TimeDelta;
use chrono::Timelike;
use colored::Colorize;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_environment::worker_environment::TacticalOptions;
use ordinator_scheduling_environment::Percent;
use priority_queue::PriorityQueue;
use rand::rng;
use rand::seq::IndexedRandom;
use tactical_solution::TacticalObjectiveValue;
use tactical_solution::TacticalScheduledOperations;
use tactical_solution::TacticalSolution;
use tracing::event;
use tracing::warn;
use tracing::Level;

use self::assert_functions::TacticalAssertions;
use self::tactical_parameters::TacticalParameters;
use self::tactical_solution::OperationSolution;

// If using a single crate, call this directly
#[derive(Debug)]
pub struct TacticalAlgorithm<Ss>(
    Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>,
)
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutions;

// TODO [ ] 2025-07-02
// The first thing to do is understand what the code is currently doing. What
// does that mean?
// * What are the loadings?
// * What is the objective value?
// * A function for each of the constraints of the problem.
// FIX
// Move the `tactical_days` into the parameters.
// QUESTION
// I think that we should delete all these, and turn the nested hashmaps into
// Vec<Vec<Work>> instead. That would be a much better solution.
// TODO [ ]
// Delete all the getters and turn the TacticalResources into a array based
// representation.
// TODO [ ]
// You have to make this thing work.
type DayIndex = usize;
impl<Ss> TacticalAlgorithm<Ss>
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
        AbLNSUtils<SolutionType = TacticalSolution>,
    Ss: SystemSolutions<Tactical = TacticalSolution>,
{
    pub fn capacity(&self, resource: &Skill, day: DayIndex) -> Result<&Work>
    {
        Ok(&self
            .parameters
            .tactical_capacity
            .resources
            .get(resource)
            .with_context(|| format!("No entry for resource{resource}"))?
            .days[day])
    }

    fn determine_aggregate_excess(&self, tactical_objective_value: &mut TacticalObjectiveValue)
    {
        let mut objective_value_from_excess = 0;
        for (resources, days) in self.parameters.tactical_capacity.resources.iter() {
            // Avoid using `Day` directly; it's a core domain model with upcoming extensions in `SchedulingEnvironement`
            let loadings = self
                .solution
                .tactical_loadings
                .resources
                .get(resources)
                .cloned()
                .unwrap_or(Days::zero_from_existing(days));

            let capacity = self
                .parameters
                .tactical_capacity
                .resources
                .get(resources)
                .cloned()
                .unwrap_or(Days::zero_from_existing(days));

            for (capacity, loading) in capacity.days.iter().zip(loadings.days.iter()) {
                let excess_capacity = loading - capacity;
                if excess_capacity > Work::from(0.0) {
                    objective_value_from_excess += excess_capacity.to_f64() as u64;
                }
            }
        }
        tactical_objective_value.resource_penalty.1 = objective_value_from_excess;
    }

    fn determine_tardiness(&mut self, tactical_objective_value: &mut TacticalObjectiveValue)
    {
        let mut objective_value_from_tardiness = 0;
        for (work_order_number, _solution) in self
            .solution
            .tactical_work_orders
            .0
            .iter()
            .filter(|(_, ts)| ts.is_tactical())
        {
            let tactical_parameter = self
                .parameters
                .tactical_work_orders
                .get(work_order_number)
                .unwrap();

            let strategic_period = &self
                .loaded_system_solution
                // TODO: Consider returning an Option instead of using ok()
                .strategic()
                .ok();

            // Determine period start date from strategic actor, which handles tardiness calculation
            let period_start_date: NaiveDate = strategic_period
                .and_then(|period| period.scheduled_task(work_order_number))
                .map(|where_is_work_order| {
                    match where_is_work_order {
                        WhereIsWorkOrder::Strategic(period) => period.start_datetime().date_naive(),
                        WhereIsWorkOrder::Tactical(period) => period.start_datetime().date_naive(),
                        // ISSUE #000 TODO [ ] 2025-07-22 fix the tactical objective
                        WhereIsWorkOrder::NotScheduled => {
                            tactical_parameter.earliest_allowed_start_date
                        }
                    }
                })
                .expect("All edge cases have been handled above");

            let mut activity_keys: Vec<ActivityNumber> = tactical_parameter
                .tactical_operation_parameters
                .keys()
                .cloned()
                .collect();

            activity_keys.sort_unstable_by(|a, b| b.cmp(a));

            let last_activity = activity_keys.last().unwrap();

            let last_day = self
                .solution
                .tactical_scheduled_days(work_order_number, last_activity)
                .expect("Missing state from the tactical agent when calculating objective value")
                .last()
                .unwrap()
                .0
                .date;

            // Calculate the difference in days, ensuring non-negative value
            let day_difference = (last_day - period_start_date).max(TimeDelta::zero());

            objective_value_from_tardiness +=
                tactical_parameter.weight * day_difference.num_days() as u64;
        }
        tactical_objective_value.urgency.1 = objective_value_from_tardiness;
    }

    fn determine_loading(&self) -> f64
    {
        let length = self.parameters.tactical_days.len();
        let mut total_capacity = Work::from(0.0);
        let mut total_loading = Work::from(0.0);

        let loadings = &self.solution.tactical_loadings;
        let capacity = &self.parameters.tactical_capacity;

        // Combine the keys
        let loading_keys = loadings.resources.keys();

        let capacity_keys = capacity.resources.keys();

        let all_keys = loading_keys.chain(capacity_keys).collect::<HashSet<_>>();
        let zero_days = Days::new(vec![Work::from(0.0); length]);

        for key in all_keys {
            total_loading += loadings
                .resources
                .get(key)
                .unwrap_or(&zero_days)
                .days
                .clone()
                .into_iter()
                .sum::<Work>();
            total_capacity += capacity
                .resources
                .get(key)
                .unwrap_or(&zero_days)
                .days
                .clone()
                .into_iter()
                .sum::<Work>();
        }

        total_loading.to_f64() / total_capacity.to_f64()
    }

    fn force_schedule_tactical_work_orders(
        &mut self,
        forced_operations: Vec<WorkOrderNumber>,
    ) -> Result<()>
    where
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
            AbLNSUtils<SolutionType = TacticalSolution>,
        Ss: SystemSolutions<Tactical = TacticalSolution>,
    {
        for work_order_number in forced_operations {
            let parameters = self
                .parameters
                .tactical_work_orders
                .get(&work_order_number)
                .context("WorkOrder not present in TacticalActor")?;

            // TODO: Create a generic forced scheduling concept for reuse
            let start_days = parameters
                .tactical_operation_parameters
                .iter()
                .map(|e| {
                    (
                        e.1.forced_start_date.clone(),
                        e.1.work_remaining,
                        e.1.operating_time,
                        e.1.number,
                    )
                })
                .collect::<Vec<_>>();

            determine_forced_tactical_assignment(&start_days);

            // let operation_solution = OperationSolution::new(scheduled,
            // resource, number, work_remaining, work_order_number,
            // activity_number)
        }

        Ok(())
    }

    fn determine_percent_scheduled(
        &self,
        tactical_objective_value: &mut TacticalObjectiveValue,
    ) -> Result<()>
    {
        let tactical_scheduled = self
            .solution
            .tactical_work_orders
            .0
            .iter()
            .filter(|(_k, v)| match v {
                WhereIsWorkOrder::Strategic(_period) => false,
                WhereIsWorkOrder::Tactical(_) => true,
                WhereIsWorkOrder::NotScheduled => false,
            })
            .count();

        let tactical_total = self.parameters.tactical_work_orders.len();

        tactical_objective_value.percent_scheduled.1 =
            Percent::new(tactical_scheduled as u64, tactical_total as u64)?;

        Ok(())
    }
}

// TODO: Add resource-specific and people count logic
#[allow(unused_assignments)]
fn determine_forced_tactical_assignment(
    scheduled_days: &[(Option<Day>, Work, Work, u64)],
) -> Vec<VecDeque<(Day, Work)>>
{
    let mut operation_index = 0;

    // Outer `Vec` is the operation, the inner `Vec` is the day for the operation
    let mut operation_day_work: Vec<VecDeque<(Day, Work)>> =
        vec![VecDeque::new(); scheduled_days.len()];
    let mut done_indices: Vec<bool> = vec![false; scheduled_days.len()];
    let mut work_in_operation = scheduled_days.iter().map(|e| e.1).collect::<Vec<_>>();
    let mut backwards = false;
    let mut current_day: Option<Day> = None;
    let mut operational_schedule_information = &scheduled_days[operation_index];

    while !done_indices.iter().all(|e| *e) {
        // Extract the current operation's schedule information
        operational_schedule_information = match &scheduled_days.get(operation_index) {
            Some(value) => value,
            None => {
                if backwards {
                    operation_index -= 1;
                } else {
                    backwards = true;
                }
                continue;
            }
        };
        dbg!(
            operation_index,
            &operational_schedule_information,
            &work_in_operation,
            &current_day,
        );

        match &current_day {
            Some(_start_day) => {
                let work =
                    work_in_operation[operation_index].min(operational_schedule_information.2);

                if work.is_zero() {
                    done_indices[operation_index] = true;
                    if backwards {
                        if operation_index == 0 {
                            break;
                        }
                        operation_index -= 1;
                        dbg!(
                            operation_index,
                            &operational_schedule_information,
                            &work_in_operation,
                            &current_day,
                            &operation_day_work,
                        );
                        current_day = operation_day_work
                            .iter()
                            .flatten()
                            .next()
                            .map(|e| e.0.clone());
                    } else {
                        operation_index += 1;
                        current_day = None;
                    }
                } else if backwards {
                    operation_day_work[operation_index]
                        .push_front((current_day.clone().unwrap(), work));
                    work_in_operation[operation_index] -= work;
                    if backwards {
                        sub_one_day(&mut current_day).unwrap();
                    } else {
                        add_one_day(&mut current_day).unwrap();
                    }
                } else {
                    operation_day_work[operation_index]
                        .push_back((current_day.clone().unwrap(), work));
                    work_in_operation[operation_index] -= work;
                    if backwards {
                        sub_one_day(&mut current_day).unwrap();
                    } else {
                        add_one_day(&mut current_day).unwrap();
                    }
                }

                dbg!(&current_day, operation_index);
            }

            // Initialize current day if not yet set
            None => match &operational_schedule_information.0 {
                // If the operation has a forced start date, use it
                Some(operation_day) => {
                    current_day = Some(operation_day.clone());
                    continue;
                }
                None => {
                    if backwards {
                    } else {
                        operation_index += 1;
                    }
                }
            },
        }
    }
    operation_day_work
}

fn add_one_day(current_day: &mut Option<Day>) -> Option<()>
{
    *current_day = Some(Day {
        day_index: current_day.as_ref().unwrap().day_index.checked_add(1)?,
        date: current_day
            .as_ref()
            .unwrap()
            .date
            .checked_add_days(chrono::Days::new(1))
            .unwrap(),
    });
    Some(())
}

fn sub_one_day(current_day: &mut Option<Day>) -> Option<()>
{
    *current_day = Some(Day {
        day_index: current_day.as_ref().unwrap().day_index.checked_sub(1)?,
        date: current_day
            .as_ref()
            .unwrap()
            .date
            .checked_sub_days(chrono::Days::new(1))
            .unwrap(),
    });
    Some(())
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for TacticalAlgorithm<Ss>
where
    Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
        AbLNSUtils<SolutionType = TacticalSolution>,
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutions<Tactical = TacticalSolution>,
{
    type Algorithm =
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;
    type Options = TacticalOptions;

    fn incorporate_system_solution(&mut self) -> Result<bool>
    {
        // TODO: Unschedule work orders that fall outside tactical_days window
        // TODO: Implement force scheduling method in ActorBasedLargeNeighborhoodSearch
        self.force_schedule()
            .context("Could not force schedule tactical solutions")?;

        warn!(target: "debug", tactical_solution = ?self.solution);
        Ok(true)
    }

    fn make_atomic_pointer_swap(&mut self)
    {
        // TODO: Optimize with COW or selective cloning of SharedSolution fields
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.tactical_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<<<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective>,
    >
    {
        let options = &self.parameters.tactical_options;

        let mut tactical_objective_value = TacticalObjectiveValue::new(options);

        self.determine_tardiness(&mut tactical_objective_value);

        // Calculate penalty for exceeding the capacity
        self.determine_aggregate_excess(&mut tactical_objective_value);

        self.determine_percent_scheduled(&mut tactical_objective_value)?;

        tactical_objective_value.aggregate_objectives();

        event!(
            Level::INFO,
            target = "optimization",
            aggregate_load = self.determine_loading()
        );

        if tactical_objective_value.objective_value < self.solution.objective_value.objective_value
        {
            Ok(ObjectiveValueType::Better(tactical_objective_value))
        } else {
            Ok(ObjectiveValueType::Worse(tactical_objective_value))
        }
    }

    fn schedule(&mut self) -> Result<()>
    {
        // TODO: Optimize performance and maintainability of scheduling logic
        self.assert_that_loading_matches_scheduled()
            .with_context(|| {
                format!(
                    "assert_that_loading_matches_scheduled\nLocation: {}",
                    Location::caller()
                )
                .bright_red()
            })?;

        self.assert_that_total_loading_is_equal_to_total_scheduled()
            .with_context(|| {
                format!(
                    "assert_that_total_loading_is_equal_to_total_scheduled\nLocation: {}",
                    Location::caller(),
                )
                .bright_red()
            })?;
        for (work_order_number, solution) in &self.solution.tactical_work_orders.0.clone() {
            let tactical_parameter = self
                .parameters
                .tactical_work_orders
                .get(work_order_number)
                .expect("TacticalParameter should ALWAYS be available for a TacticalSolution")
                .clone();

            // All the work orders that does not have a solution gets pushed to the queue.
            if matches!(solution, WhereIsWorkOrder::NotScheduled) {
                self.solution_intermediate
                    .push(*work_order_number, tactical_parameter.weight);
            }
        }

        let mut start_day_index = 0;

        let mut loop_state: LoopState = LoopState::Unscheduled;

        let mut current_work_order_number = match self.solution_intermediate.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => return Ok(()),
        };

        let mut counter = 0;

        'back_to_loop_state_handle: loop {
            counter += 1;

            event!(
                Level::DEBUG,
                main_loop_counter = counter,
                start_day_index = start_day_index,
                priority_queue_len = self.solution_intermediate.len(),
            );

            event!(target: "developer", Level::WARN, start_day_index);
            let tactical_parameter = match loop_state {
                LoopState::Unscheduled => {
                    start_day_index += 1;

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::Scheduled => {
                    start_day_index = 0;

                    current_work_order_number = match self.solution_intermediate.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(target: "developer", Level::INFO, "main_loop break: FINISH ON LoopState::Scheduled");
                            break;
                        }
                    };

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::ReleaseFromTactical => {
                    self.solution
                        .tactical_work_orders
                        .0
                        .insert(current_work_order_number, WhereIsWorkOrder::NotScheduled);

                    start_day_index = 0;

                    current_work_order_number = match self.solution_intermediate.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(target: "developer", Level::INFO, "main_loop break: FINISH ON LoopState::Scheduled");
                            break;
                        }
                    };

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
            };

            let mut operation_solutions = TacticalScheduledOperations::default();

            let allowed_starting_days = self
                .parameters
                .tactical_days
                .iter()
                .filter(|day| tactical_parameter.earliest_allowed_start_date <= day.date)
                .nth(start_day_index);

            let Some(start_day) = allowed_starting_days else {
                loop_state = LoopState::ReleaseFromTactical;
                continue 'back_to_loop_state_handle;
            };

            let mut current_day = self
                .parameters
                .tactical_days
                .iter()
                .filter(|date| start_day.date <= date.date)
                .peekable();

            for activity_number in tactical_parameter.tactical_operation_parameters.keys() {
                let operation_parameters = tactical_parameter
                    .tactical_operation_parameters
                    .get(activity_number)
                    .expect("The work order should always have its corresponding parameters");

                let current_day_peek = match current_day.peek() {
                    Some(day) => day,
                    None => {
                        loop_state = LoopState::ReleaseFromTactical;
                        continue 'back_to_loop_state_handle;
                    }
                };

                let first_day_remaining_capacity = match self
                    .remaining_capacity(&operation_parameters.resource, current_day_peek)
                {
                    Some(remaining_capacity) => remaining_capacity,
                    None => {
                        loop_state = LoopState::Unscheduled;
                        continue 'back_to_loop_state_handle;
                    }
                };

                let loadings = determine_load(
                    first_day_remaining_capacity,
                    &operation_parameters.operating_time,
                    operation_parameters.work_remaining,
                );

                let mut activity_load = Vec::<(Day, Work)>::new();

                for load in loadings {
                    let day = match current_day.peek() {
                        Some(day) => (*day).clone(),
                        None => {
                            break;
                        }
                    };

                    activity_load.push((day, load));

                    current_day.next();

                    let peek_next_day = current_day.peek();
                    let current_day = match peek_next_day {
                        Some(next_day) => next_day,
                        None => {
                            break;
                        }
                    };

                    if self
                        .remaining_capacity(&operation_parameters.resource, current_day)
                        .is_none()
                    {
                        loop_state = LoopState::Unscheduled;
                        continue 'back_to_loop_state_handle;
                    };
                }

                let calculated_tactical_work =
                    activity_load.iter().fold(Work::from(0.0), |mut acc, e| {
                        acc += e.1;
                        acc
                    });

                if operation_parameters.work_remaining != calculated_tactical_work {
                    loop_state = LoopState::ReleaseFromTactical;
                    continue 'back_to_loop_state_handle;
                }

                ensure!(
                    operation_parameters.work_remaining == calculated_tactical_work,
                    "required_work: {}\noptimized_work: {}\nwork_order: {}\nactivity: {}\nnext day: {:?}",
                    operation_parameters.work_remaining,
                    calculated_tactical_work,
                    current_work_order_number,
                    activity_number,
                    (current_day.peek()).clone(),
                );
                let operation_solution = OperationSolution::new(
                    activity_load,
                    operation_parameters.resource,
                    operation_parameters.number,
                    operation_parameters.work_remaining,
                    current_work_order_number,
                    *activity_number,
                );

                operation_solutions.insert_operation_solution(*activity_number, operation_solution);
            }

            self.update_loadings(&operation_solutions, LoadOperation::Add)?;

            loop_state = LoopState::Scheduled;

            self.solution
                .tactical_insert_work_order(current_work_order_number, operation_solutions);
        }
        Ok(())
    }

    // TODO: Reduce state duplication in scheduling implementation
    // TODO: Verify `schedule` method is the issue source
    // TODO: Implement `schedule_forced` method for forced work order handling
    fn unschedule(&mut self) -> Result<()>
    {
        let mut rng = rng();
        let work_order_numbers: Vec<WorkOrderNumber> = self
            .solution
            .tactical_work_orders
            .0
            .clone()
            .into_keys()
            .collect();

        let random_work_order_numbers = work_order_numbers.choose_multiple(
            &mut rng,
            self.parameters
                .tactical_options
                .number_of_removed_work_orders,
        );

        // Log work order count for debugging
        event!(target: "developer", Level::INFO, number_of_work_orders_in_tactical_solution = self.solution.tactical_work_orders.0.values().filter(|e| matches!(e, WhereIsWorkOrder::Tactical(_))).count());
        for work_order_number in random_work_order_numbers {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule tactical work order: {:?}\n\
                        Location: {}",
                        work_order_number,
                        Location::caller(),
                    )
                })?;
        }
        event!(target: "developer", Level::INFO, number_of_work_orders_in_tactical_solution = self.solution.tactical_work_orders.0.values().filter(|e| matches!(e, WhereIsWorkOrder::Tactical(_))).count());
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm
    {
        &mut self.0
    }

    fn force_schedule(&mut self) -> Result<()>
    {
        let forced_work_orders: Vec<_> = self
            .parameters
            .tactical_work_orders
            .iter()
            .filter(|e| {
                e.1.tactical_operation_parameters
                    .iter()
                    .any(|e| e.1.forced_start_date.is_some())
            })
            .map(|e| e.0)
            .cloned()
            .collect();

        self.force_schedule_tactical_work_orders(forced_work_orders)
    }

    fn throttling(&self, throttling: &ordinator_configuration::throttling::Throttling) -> u64
    {
        throttling.tactical_throttling
    }
}

enum LoopState
{
    Unscheduled,
    Scheduled,
    ReleaseFromTactical,
}
impl<Ss> Deref for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target =
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}
impl<Ss> DerefMut for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl<Ss> TacticalAlgorithm<Ss>
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutions,
{
    fn update_loadings(
        &mut self,
        operation_solutions: &TacticalScheduledOperations,
        load_operation: LoadOperation,
    ) -> Result<()>
    {
        for operation in operation_solutions.0.values() {
            let resource = &operation.resource;
            for loadings in &operation.scheduled {
                let day = &loadings.0;
                let load = &loadings.1;
                let resource_loading = self
                    .solution
                    .tactical_loadings
                    .get_resource(resource, day.day_index)?;

                let new_load = match load_operation {
                    LoadOperation::Add => resource_loading + load,
                    LoadOperation::Sub => resource_loading - load,
                };
                *self
                    .solution
                    .tactical_loadings
                    // WARN [ ] 2025-07-02 It is crucial that the index is always at the right place
                    // in the code.
                    .get_resource_mut(resource, day.day_index)? = new_load;
            }
        }
        Ok(())
    }

    pub fn unschedule_specific_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Result<()>
    {
        let previous_solution = self
            .solution
            .tactical_work_orders
            .0
            .insert(work_order_number, WhereIsWorkOrder::NotScheduled)
            .context("This means that the TacticalAlgorithm has been initialized wrong")?;

        match previous_solution {
            WhereIsWorkOrder::Strategic(_) => Ok(()),
            WhereIsWorkOrder::Tactical(operation_solutions) => {
                self.update_loadings(&operation_solutions.clone(), LoadOperation::Sub)
            }
            WhereIsWorkOrder::NotScheduled => Ok(()),
        }
    }

    fn remaining_capacity(&self, resource: &Skill, day: &Day) -> Option<Work>
    {
        let remaining_capacity = self
            .parameters
            .tactical_capacity
            .get_resource(resource, day.day_index)
            .ok()?
            - self
                .solution
                .tactical_loadings
                .get_resource(resource, day.day_index)
                .ok()?;

        if remaining_capacity <= Work::from(0.0) {
            None
        } else {
            Some(remaining_capacity)
        }
    }
}
fn determine_load(
    remaining_capacity: Work,
    operating_time: &Work,
    mut work_remaining: Work,
) -> Vec<Work>
{
    let mut loadings = Vec::new();

    let first_day_load = match remaining_capacity.partial_cmp(operating_time) {
        Some(Ordering::Less) => remaining_capacity,
        Some(Ordering::Equal) => remaining_capacity,
        Some(Ordering::Greater) => *operating_time,
        None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
    }.min(work_remaining);

    loadings.push(first_day_load);
    work_remaining -= first_day_load;

    while work_remaining > Work::from(0.0) {
        let load = *operating_time.min(&work_remaining);
        loadings.push(load);
        work_remaining -= load;
    }
    loadings
}

#[allow(dead_code)]
enum OperationDifference
{
    SameDay,
    DiffDay,
}
impl<Ss>
    From<Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>>
    for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(
        value: Algorithm<
            TacticalSolution,
            TacticalParameters,
            PriorityQueue<WorkOrderNumber, u64>,
            Ss,
        >,
    ) -> Self
    {
        TacticalAlgorithm(value)
    }
}

#[cfg(test)]
pub mod tests
{
    use chrono::NaiveDate;
    use ordinator_scheduling_environment::time_environment::day::Day;
    use ordinator_scheduling_environment::work_order::operation::Work;
    use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;

    use super::determine_forced_tactical_assignment;
    use crate::algorithm::determine_load;

    #[test]
    fn test_determine_load_1()
    {
        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(5.0);
        let work_remaining = Work::from(10.0);

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(
            loadings,
            vec![Work::from(3.0), Work::from(5.0), Work::from(2.0)]
        );
    }

    #[test]
    fn test_determine_load_2()
    {
        let _id = ActorCompositeId::default();

        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(
            loadings,
            vec![
                Work::from(3.0),
                Work::from(3.0),
                Work::from(3.0),
                Work::from(1.0)
            ]
        );
    }

    #[test]
    fn test_work_min()
    {
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let min_work = operating_time.min(work_remaining);

        assert_eq!(min_work, Work::from(3.0));

        let operating_time = Work::from(12.0);
        let work_remaining = Work::from(10.0);

        let min_work = operating_time.min(work_remaining);

        assert_eq!(min_work, Work::from(10.0));
    }

    // Integration tests preferred; objective value depends on other Solutions
    #[test]
    fn test_determine_forced_assignment_1()
    {
        let day = Day::new(3, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let scheduled_days = vec![(Some(day.clone()), Work::from(4.0), Work::from(6.0), 1)];
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(value, vec![vec![(day.clone(), Work::from(4.0))]])
    }

    #[test]
    fn test_determine_forced_assignment_2()
    {
        let day = Day::new(3, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day::new(4, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
        let scheduled_days = vec![(Some(day.clone()), Work::from(8.0), Work::from(6.0), 1)];
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(
            value,
            vec![vec![
                (day.clone(), Work::from(6.0)),
                (day_2.clone(), Work::from(2.0))
            ]]
        )
    }

    #[test]
    fn test_determine_forced_assignment_3()
    {
        let day_1 = Day::new(3, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day::new(4, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());

        let scheduled_days = vec![
            (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
            (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
        ];
        // TODO [ ] - Add the Date
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(
            value,
            vec![
                vec![
                    (day_1.clone(), Work::from(6.0)),
                    (day_2.clone(), Work::from(2.0))
                ],
                vec![(day_2.clone(), Work::from(4.0))]
            ]
        )
    }

    #[test]
    fn test_determine_forced_assignment_4()
    {
        let day_1 = Day::new(1, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day::new(2, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());

        let scheduled_days = vec![
            (None, Work::from(6.0), Work::from(6.0), 1),
            (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
            (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
        ];
        // TODO [ ] - Add the Date
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(
            value,
            vec![
                vec![(day_1.clone(), Work::from(6.0))],
                vec![
                    (day_1.clone(), Work::from(6.0)),
                    (day_2.clone(), Work::from(2.0))
                ],
                vec![(day_2.clone(), Work::from(4.0))]
            ]
        )
    }
    // #[test]
    // fn test_determine_forced_assignment_5()
    // {
    //     let day_0 = Day::new(0, NaiveDate::from_ymd_opt(2024, 12,
    // 31).unwrap());     let day_1 = Day::new(1,
    // NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());     let day_2 =
    // Day::new(2, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());

    //     let scheduled_days = vec![
    //         (None, Work::from(12.0), Work::from(6.0), 1),
    //         (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
    //         (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
    //     ];
    //     // TODO [ ] - Add the Date
    //     let value = determine_forced_tactical_assignment(&scheduled_days);

    //     assert_eq!(value, vec![
    //         vec![
    //             (day_0.clone(), Work::from(6.0)),
    //             (day_1.clone(), Work::from(6.0))
    //         ],
    //         vec![
    //             (day_1.clone(), Work::from(6.0)),
    //             (day_2.clone(), Work::from(2.0))
    //         ],
    //         vec![(day_2.clone(), Work::from(4.0))]
    //     ])
    // }
    // #[test]
    // fn test_determine_forced_assignment_6()
    // {
    //     let day_0 = Day::new(0, NaiveDate::from_ymd_opt(2024, 12,
    // 30).unwrap());     let day_1 = Day::new(1,
    // NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());     let day_2 =
    // Day::new(2, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    //     let day_3 = Day::new(3, NaiveDate::from_ymd_opt(2025, 1,
    // 2).unwrap());

    //     let scheduled_days = vec![
    //         (None, Work::from(18.0), Work::from(6.0), 1),
    //         (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
    //         (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
    //     ];
    //     // TODO [ ] - Add the Date
    //     let value = determine_forced_tactical_assignment(&scheduled_days);

    //     assert_eq!(value, vec![
    //         vec![
    //             (day_0.clone(), Work::from(6.0)),
    //             (day_1.clone(), Work::from(6.0)),
    //             (day_2.clone(), Work::from(6.0)),
    //         ],
    //         vec![
    //             (day_2.clone(), Work::from(6.0)),
    //             (day_3.clone(), Work::from(2.0))
    //         ],
    //         vec![(day_3.clone(), Work::from(4.0))]
    //     ])
    // }
}
