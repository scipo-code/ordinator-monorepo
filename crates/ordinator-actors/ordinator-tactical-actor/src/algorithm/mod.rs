pub mod assert_functions;
mod tactical_fixtures;
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

use anyhow::Context;
use anyhow::Result;
use chrono::NaiveDate;
use chrono::TimeDelta;
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
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::TacticalOptions;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use priority_queue::PriorityQueue;
use rand::rng;
use rand::seq::IndexedRandom;
use tactical_solution::TacticalObjectiveValue;
use tactical_solution::TacticalScheduledOperations;
use tactical_solution::TacticalSolution;
use tracing::Level;
use tracing::event;
use tracing::warn;

use self::assert_functions::TacticalAssertions;
use self::tactical_parameters::TacticalParameters;
use self::tactical_solution::OperationSolution;

// If you had a single crate you should simply call thie
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
    Ss: SystemSolutions,
{
    pub fn capacity(&self, resource: &Resources, day: DayIndex) -> Result<&Work>
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
            // You do not want to use the `Day` here. Remember that the `Day` is a core
            // domain model. And it should be treated as such. There are many additions
            // coming up into the [`SchedulingEnvironement`].
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
                // This should be an option instead
                .strategic()
                .ok();

            // You need a thoughtful way of handling this. I think that the best approch
            // here is to make the
            //
            // ESSAY: Do you want the Tactical here? Ahh you are in the
            // I think that the Tactical period should not make any sense here.
            // Here you are relying on the StrategicActor to determine the
            // tardiness. That is actually fine I think, it simply means that
            // you have to take the
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
                .0;

            // Ahh this is completely wrong again. I think that the best approach here is to
            // make the system work.
            let day_difference = (last_day - period_start_date).max(TimeDelta::zero());

            objective_value_from_tardiness +=
                tactical_parameter.weight * day_difference.num_days() as u64;
        }
        tactical_objective_value.urgency.1 = objective_value_from_tardiness;
    }

    fn determine_loading(&self) -> f64
    where
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
            AbLNSUtils<SolutionType = TacticalSolution>,
        Ss: SystemSolutions<Tactical = TacticalSolution>,
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

            // TODO [x] - break, consider if you are going in the right direction here
            // TODO [ ] - Make a generic forced concept in the code.
            // We need a common expression here to make the code work as expected.
            //
            // You want to put this into the,
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
}

// Essay should you fix this now? Or make a new endpoint?
// TODO [ ] - add different resources between operation specific logic
// TODO [ ] - add number_of_people logic
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
        // This is to extract the correct
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

            // If the current day is not defined. We should start from here.
            None => match &operational_schedule_information.0 {
                // Some if the operation is forced to a particular day.
                Some(operation_day) => {
                    // A forced operation should always override
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
    *current_day = Some(Day(current_day
        .as_ref()
        .unwrap()
        .0
        .checked_add_days(chrono::Days::new(1))
        .unwrap()));
    Some(())
}

fn sub_one_day(current_day: &mut Option<Day>) -> Option<()>
{
    *current_day = Some(Day(current_day
        .as_ref()
        .unwrap()
        .0
        .checked_sub_days(chrono::Days::new(1))
        .unwrap()));
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
        // Here we have to
        // TODO [ ] 2025-07-17 make the `incorporate_system_solution`
        // ESSAY [ ] 2025-07-17
        // What should be done here?
        // When the `UnloadingPoint` leaves the tactical schedule it should
        // be forced out of the solution. So we the work order is Unscheduled
        // and the work order is not in the tactical point of view we should
        // make the code unschedule the WorkOrder if the new date is outside
        // of the `tactical_days`. Yes that is the way of coding this system.
        // First two TEST.
        // [ ] Does scheduling inside of the `tactical_days` also give an error?
        // -> No it does not, that means that it is only when the work order
        //    leaves the tactical_days that we have an issue
        // [ ] Does removing the `assert` in the `schedule` function remove the error?
        // ESSAY: Should the schedule handle this? No! I think that we need a very
        // clear policy here.
        //
        // This will be crucial, the tactical have to respect the `TechnicianActor` as
        // well. I think that the best approach is to make the system. Work with
        // that, but now the goal is to make Brian happy.
        // for (work_order_number, tactical_parameter) in
        // &self.parameters.tactical_work_orders {

        // tactical_parameter.

        // Now we want to remove the work orders that are no longer to be a
        // part of the solution.
        // }
        //

        // ISSUE #000 - make a method for force scheduling in the
        // ActorBasedLargeNeighborhoodSearch.
        // TODO [ ] Every actor needs this function. Is that correct? Yes! Every single
        // Now? Yes.
        //
        // for operation in forced_operations {
        //     self.force_schedule(forced_operations);
        // }
        //
        self.force_schedule()
            .context("Could not force schedule tactical solutions")?;

        warn!(target: "debug", tactical_solution = ?self.solution);
        Ok(true)
    }

    fn make_atomic_pointer_swap(&mut self)
    {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
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

        tactical_objective_value.aggregate_objectives();

        // What is it that you actually want to test here? I am not sure, you need to
        // get an understanding of what the system is doing. That is the most
        // crucial. Remember
        // 1. Knowledge
        // 2. Tools
        // 3. Harness and tests
        // 4. Source code
        // 5. Runtime
        // You are currently lacking the knowledge of why this system is not performing
        // as it should. And that is the major problem here. You have several
        // options here and it is crucial that you know which one to use:
        // 1. tracing
        // 2. debugger
        // 3. error, assertion, and constraint handling
        // 4. tester thread
        // 5. Swagger pinging
        //
        // Take your time here. It is crucial that you spend your time getting this
        // correct What other tools do you have available here?

        event!(
            Level::INFO,
            target = "optimization",
            aggregate_load = self.determine_loading()
        );

        if tactical_objective_value.objective_value < self.solution.objective_value.objective_value
        {
            event!(Level::INFO, tactical_objective_value_better = ?tactical_objective_value);
            Ok(ObjectiveValueType::Better(tactical_objective_value))
        } else {
            event!(Level::INFO, tactical_objective_value_worse = ?tactical_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()>
    {
        // The code here is all wrong. It is not as performant as it should be
        // and it is not as maintainable as it could be. It has been
        // designed for being understandable and that is also good.
        //
        // Just be aware of the issue.

        // I am not sure whether this assertion should even be here/
        // ESSAY on whether the `incorporate_state` or `schedule` and `unschedule`
        // should handle the changes to the [`SchedulingEnvironment`].
        //
        // What part of the program should be responsible for all of this? I believe
        // that the fundamental issue here is that we should be able to make
        // something that can change the way that we work with the.
        self.asset_that_loading_matches_scheduled()
            .with_context(|| format!("TESTING_ASSERTION\nLocation: {}", Location::caller()))?;

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
        // The issue is that the code here is running a lot of iterations. What should
        // we do about this? I am not really sure! I thi

        'back_to_loop_state_handle: loop {
            counter += 1;

            event!(
                Level::DEBUG,
                main_loop_counter = counter,
                start_day_index = start_day_index,
                priority_queue_len = self.solution_intermediate.len(),
            );

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
                            event!(Level::DEBUG, "main_loop break");
                            break;
                        }
                    };

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::ReleasedFromTactical => {
                    self.solution
                        .release_from_tactical_solution(&current_work_order_number);

                    start_day_index = 0;

                    current_work_order_number = match self.solution_intermediate.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(Level::DEBUG, "main_loop break");
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

            let mut all_days = self.parameters.tactical_days.clone();

            let allowed_starting_days: Vec<&Day> = self
                .parameters
                .tactical_days
                .iter()
                .filter(|day| tactical_parameter.earliest_allowed_start_date <= day.0)
                .collect();

            let start_day: Day = match allowed_starting_days.get(start_day_index) {
                Some(start_day) => (*start_day).clone(),
                None => {
                    loop_state = LoopState::ReleasedFromTactical;
                    continue 'back_to_loop_state_handle;
                }
            };

            let allowed_days: Vec<_> = all_days
                .iter_mut()
                .filter(|date| start_day.0 <= date.0)
                .collect();

            let mut current_day = allowed_days.into_iter().enumerate().peekable();

            let mut sorted_activities = tactical_parameter
                .tactical_operation_parameters
                .keys()
                .clone()
                .collect::<Vec<&ActivityNumber>>();

            sorted_activities.sort();

            for activity in sorted_activities {
                let operation_parameters = tactical_parameter
                    .tactical_operation_parameters
                    .get(activity)
                    .expect("The work order should always have its corresponding parameters");

                let resource = operation_parameters.resource;

                let current_day_peek = match current_day.peek() {
                    Some(day) => day,
                    None => {
                        loop_state = LoopState::ReleasedFromTactical;
                        continue 'back_to_loop_state_handle;
                    }
                };

                let first_day_remaining_capacity =
                    match self.remaining_capacity(&resource, current_day_peek.0) {
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
                // The breaks here mean that the code might input a partial work order
                // This should not matter for correctness.

                for load in loadings {
                    let day = match current_day.peek() {
                        Some(day) => (*day.1).clone(),
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

                    if self.remaining_capacity(&resource, current_day.0).is_none() {
                        loop_state = LoopState::Unscheduled;
                        continue 'back_to_loop_state_handle;
                    };
                }

                let operation_solution = OperationSolution::new(
                    activity_load,
                    resource,
                    operation_parameters.number,
                    operation_parameters.work_remaining,
                    current_work_order_number,
                    *activity,
                );

                operation_solutions.insert_operation_solution(*activity, operation_solution);
            }

            self.update_loadings(&operation_solutions, LoadOperation::Add)?;
            loop_state = LoopState::Scheduled;

            self.solution
                .tactical_insert_work_order(current_work_order_number, operation_solutions);
            self.asset_that_loading_matches_scheduled()
                .with_context(|| {
                    format!("TESTING_ASSERTION\nfile: {}\nline: {}", file!(), line!())
                })?;
        }
        Ok(())
    }

    // So what should be checked to understand this?
    // We know that the error is not in the creation of the parameters
    //
    // TODO [ ]
    // Confirm that it is the `schedule` method that is causing the issue
    // Remember that you should only change one thing at a time.
    // Where do we want to fix this? The change is not found in the Strategic
    // Actor alone. That means that it should not be in the
    // `incorporate_system_solution` although that function also needs to be
    // implemented correctly.
    //
    // What other options do we have here? I think that the best approach is
    // to make the system work correctly with the. You are not the most focussed at
    // the moment. What should you do here? I think that the best approach is to
    // make the system work correctly.
    //
    // QUESTION [ ] 2025-07-17 where are the forced work orders handled in the
    // tactical actor?
    //
    // Okay so the TacticalActor does not have an `schedule_forced` that
    // makes this really difficult to work with.
    //
    // How does the parameters even fit in together with the idea that the
    // code should be working with the earliest_start_day to put the
    // work_order on the correct day?
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

        // How can you make something that will allow us to catch the
        // error instantneously?
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
}

enum LoopState
{
    Unscheduled,
    Scheduled,
    ReleasedFromTactical,
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
            for (day_index, day_work) in operation.scheduled.iter().enumerate() {
                let load = &day_work.1;
                let resource_loading = self
                    .solution
                    .tactical_loadings
                    .get_resource(resource, day_index)?;

                let new_load = match load_operation {
                    LoadOperation::Add => resource_loading + load,
                    LoadOperation::Sub => resource_loading - load,
                };
                *self
                    .solution
                    .tactical_loadings
                    // WARN [ ] 2025-07-02 It is crucial that the index is always at the right place
                    // in the code.
                    .get_resource_mut(resource, day_index)? = new_load;
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

    fn remaining_capacity(&self, resource: &Resources, day_index: usize) -> Option<Work>
    {
        let remaining_capacity = self
            .parameters
            .tactical_capacity
            .get_resource(resource, day_index)
            .ok()?
            - self
                .solution
                .tactical_loadings
                .get_resource(resource, day_index)
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
    use ordinator_actor_core::algorithm::Algorithm;
    use ordinator_orchestrator_actor_traits::OperationalInterface;
    use ordinator_orchestrator_actor_traits::Solution;
    use ordinator_orchestrator_actor_traits::StrategicInterface;
    use ordinator_orchestrator_actor_traits::SupervisorInterface;
    use ordinator_orchestrator_actor_traits::SystemSolutions;
    use ordinator_orchestrator_actor_traits::TacticalInterface;
    use ordinator_scheduling_environment::time_environment::day::Day;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    use ordinator_scheduling_environment::work_order::operation::Work;
    use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
    use priority_queue::PriorityQueue;

    use super::determine_forced_tactical_assignment;
    use super::tactical_fixtures::tactical_parameters_1;
    use super::tactical_parameters::TacticalParameters;
    use super::tactical_solution::TacticalSolution;
    use crate::algorithm::determine_load;

    #[test]
    fn test_determine_load_1()
    {
        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(5.0);
        let work_remaining = Work::from(10.0);

        // You need a `SharedSolution` to call this function... Does this even
        // make anysense? I think that it does but this is going to be really really
        // difficult. You should strive to make the code here
        // This sure is a horrible trait. I think that there is a way of making this
        // in a really good way
        //
        // You could also simply remove the function from the `Algorithm inplementation`

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(loadings, vec![
            Work::from(3.0),
            Work::from(5.0),
            Work::from(2.0)
        ]);
    }

    #[test]
    fn test_determine_load_2()
    {
        let _id = ActorCompositeId::default();

        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(loadings, vec![
            Work::from(3.0),
            Work::from(3.0),
            Work::from(3.0),
            Work::from(1.0)
        ]);
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

    // You should test all this in the right order. I think that...
    // QUESTION
    // Is it correct of you to move this into the integration testing? Yes
    // absolutely. I do not see anyother way, as the `objective value` may
    // always be dependent on the other `Solution`s.
    // GOOD a decision was made here.

    #[test]
    fn test_determine_forced_assignment_1()
    {
        let day = Day(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let scheduled_days = vec![(Some(day.clone()), Work::from(4.0), Work::from(6.0), 1)];
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(value, vec![vec![(day.clone(), Work::from(4.0))]])
    }

    #[test]
    fn test_determine_forced_assignment_2()
    {
        let day = Day(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
        let scheduled_days = vec![(Some(day.clone()), Work::from(8.0), Work::from(6.0), 1)];
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(value, vec![vec![
            (day.clone(), Work::from(6.0)),
            (day_2.clone(), Work::from(2.0))
        ]])
    }

    #[test]
    fn test_determine_forced_assignment_3()
    {
        let day_1 = Day(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());

        let scheduled_days = vec![
            (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
            (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
        ];
        // TODO [ ] - Add the Date
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(value, vec![
            vec![
                (day_1.clone(), Work::from(6.0)),
                (day_2.clone(), Work::from(2.0))
            ],
            vec![(day_2.clone(), Work::from(4.0))]
        ])
    }

    #[test]
    fn test_determine_forced_assignment_4()
    {
        let day_1 = Day(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let day_2 = Day(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());

        let scheduled_days = vec![
            (None, Work::from(6.0), Work::from(6.0), 1),
            (Some(day_1.clone()), Work::from(8.0), Work::from(6.0), 1),
            (Some(day_2.clone()), Work::from(4.0), Work::from(6.0), 1),
        ];
        // TODO [ ] - Add the Date
        let value = determine_forced_tactical_assignment(&scheduled_days);

        assert_eq!(value, vec![
            vec![(day_1.clone(), Work::from(6.0))],
            vec![
                (day_1.clone(), Work::from(6.0)),
                (day_2.clone(), Work::from(2.0))
            ],
            vec![(day_2.clone(), Work::from(4.0))]
        ])
    }
    // #[test]
    // fn test_determine_forced_assignment_5()
    // {
    //     let day_0 = Day(NaiveDate::from_ymd_opt(2024, 12,
    // 31).unwrap());     let day_1 = Day()    // NaiveDate::from_ymd_opt(2025, 1,
    // 1).unwrap());     let day_2 = Day(NaiveDate::from_ymd_opt(2025, 1,
    // 2).unwrap());

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
    //     let day_0 = Day(NaiveDate::from_ymd_opt(2024, 12,
    // 30).unwrap());     let day_1 = Day()    // NaiveDate::from_ymd_opt(2024, 12,
    // 31).unwrap());     let day_2 = Day(NaiveDate::from_ymd_opt(2025, 1,
    // 1).unwrap());     let day_3 = Day(NaiveDate::from_ymd_opt(2025, 1,
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
    //
    //
    #[test]
    fn test_tactical_schedule_function()
    {
        let tactical_parameters = tactical_parameters_1();

        let tactical_algorithm = Algorithm::<
            TacticalSolution,
            TacticalParameters,
            PriorityQueue<WorkOrderNumber, u64>,
            TestSystemSolution<Operational, Strategic, Supervisor, Tactical>,
        >::builder();
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Operational;
    impl Solution for Operational
    {
        type Objective = ();
        type Parameters = ();

        fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
        {
            todo!()
        }

        fn update_objective(&mut self, other_objective: Self::Objective)
        {
            todo!()
        }
    }
    impl OperationalInterface for Operational
    {
        fn marginal_fitness_for_operational_actor<'a>(
            &'a self,
            work_order_activity: &ordinator_scheduling_environment::work_order::WorkOrderActivity,
        ) -> Option<&'a ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness>
        {
            todo!()
        }

        fn scheduled_activities_for_operational_actor(
            &self,
        ) -> std::collections::HashSet<
            ordinator_scheduling_environment::work_order::WorkOrderActivity,
        >
        {
            todo!()
        }
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Strategic;
    impl StrategicInterface for Strategic{
        
    } 
    impl Solution for Strategic
    {
        type Objective = ();
        type Parameters = ();

        fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
        {
            todo!()
        }

        fn update_objective(&mut self, other_objective: Self::Objective)
        {
            todo!()
        }
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Tactical;
    impl TacticalInterface for Tactical{
        
    } 
    impl Solution for Tactical
    {
        type Objective = ();
        type Parameters = ();

        fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
        {
            todo!()
        }

        fn update_objective(&mut self, other_objective: Self::Objective)
        {
            todo!()
        }
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Supervisor;
    impl SupervisorInterface for Supervisor{
        
    } 
    impl Solution for Supervisor
    {
        type Objective = ();
        type Parameters = ();

        fn from_parameters(parameters: &Self::Parameters) -> anyhow::Result<Self>
        {
            todo!()
        }

        fn update_objective(&mut self, other_objective: Self::Objective)
        {
            todo!()
        }
    }

    #[derive(Clone)]
    struct TestSystemSolution<Operational, Strategic, Supervisor, Tactical>
    where
        Operational: Solution + OperationalInterface,
        Strategic: Solution + StrategicInterface,
        Tactical: Solution + TacticalInterface,
        Supervisor: Solution + SupervisorInterface,
    {
        operational: Operational,
        strategic: Strategic,
        supervisor: Supervisor,
        tactical: Tactical,
    }

    impl<Operational, Strategic, Supervisor, Tactical> SystemSolutions
        for TestSystemSolution<Operational, Strategic, Supervisor, Tactical>
    where
        Operational: Solution + OperationalInterface,
        Strategic: Solution + StrategicInterface,
        Tactical: Solution + TacticalInterface,
        Supervisor: Solution + SupervisorInterface,
    {
        type Operational = Operational;
        type Strategic = Strategic;
        type Supervisor = Supervisor;
        type Tactical = Tactical;

        fn new() -> Self
        {
            todo!()
        }

        fn strategic(&self) -> anyhow::Result<&Self::Strategic>
        {
            todo!()
        }

        fn strategic_swap(
            &mut self,
            id: &ActorCompositeId,
            solution: ordinator_orchestrator_actor_traits::SolutionState<Self::Strategic>,
        ) where
            Self::Strategic: ordinator_orchestrator_actor_traits::Solution,
        {
            todo!()
        }

        fn tactical_actor_solution(&self) -> anyhow::Result<&Self::Tactical>
        {
            todo!()
        }

        fn tactical_swap(
            &mut self,
            id: &ActorCompositeId,
            solution: ordinator_orchestrator_actor_traits::SolutionState<Self::Tactical>,
        ) where
            Self::Tactical: ordinator_orchestrator_actor_traits::Solution,
        {
            todo!()
        }

        fn supervisor_actor_solutions(&self) -> anyhow::Result<&Self::Supervisor>
        {
            todo!()
        }

        fn supervisor_swap(
            &mut self,
            id: &ActorCompositeId,
            solution: ordinator_orchestrator_actor_traits::SolutionState<Self::Supervisor>,
        ) where
            Self::Supervisor: ordinator_orchestrator_actor_traits::Solution,
        {
            todo!()
        }

        fn operational_actor_solutions(
            &self,
            id: &ActorCompositeId,
        ) -> anyhow::Result<&Self::Operational>
        {
            todo!()
        }

        fn all_operational(&self) -> std::collections::HashSet<ActorCompositeId>
        {
            todo!()
        }

        fn operational_swap(
            &mut self,
            id: &ActorCompositeId,
            solution: ordinator_orchestrator_actor_traits::SolutionState<Self::Operational>,
        ) where
            Self::Operational: ordinator_orchestrator_actor_traits::Solution,
        {
            todo!()
        }
    }
}
