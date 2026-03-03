pub mod assert_functions;
pub mod weekly_parameters;
pub mod weekly_resources;
pub mod weekly_solution;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Inspect;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::ProjectInterface;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::Percent;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::WeeklyOptions;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use priority_queue::PriorityQueue;
use rand::distr::weighted::Weight;
use rand::seq::IndexedRandom;
use strum::IntoEnumIterator;
use tracing::instrument;
use tracing::warn;
use weekly_parameters::WeeklyClustering;
use weekly_parameters::WeeklyParameters;
use weekly_resources::WeeklyResources;
use weekly_solution::WeeklyObjectiveValue;
use weekly_solution::WeeklySolution;

use crate::messages::requests::WeeklyRequestResource;
use crate::messages::requests::WeeklyRequestScheduling;
use crate::messages::responses::WeeklyResponseResources;
use crate::messages::responses::WeeklyResponseScheduling;

// How would this look if made generic? impl Algorithm<WeeklySolution,
// WeeklyParameters, WeeklyAssertions> { } Note: Making behavior generic is
// important, as changes would be needed in 4 places with the current design.

#[derive(Debug)]
pub struct WeeklyAlgorithm<Ss>(
    pub  Algorithm<
        WeeklySolution,
        WeeklyParameters,
        PriorityQueue<WorkOrderNumber, i64>,
        WeeklyOptions,
        Ss,
    >,
)
where
    WeeklySolution: Solution,
    WeeklyParameters: Parameters,
    Ss: SystemSolutions,
    Algorithm<
        WeeklySolution,
        WeeklyParameters,
        PriorityQueue<WorkOrderNumber, i64>,
        WeeklyOptions,
        Ss,
    >: AbLNSUtils;

impl<Ss> Deref for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target = Algorithm<
        WeeklySolution,
        WeeklyParameters,
        PriorityQueue<WorkOrderNumber, i64>,
        WeeklyOptions,
        Ss,
    >;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}
impl<Ss> DerefMut for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for WeeklyAlgorithm<Ss>
where
    Algorithm<
        WeeklySolution,
        WeeklyParameters,
        PriorityQueue<WorkOrderNumber, i64>,
        WeeklyOptions,
        Ss,
    >: AbLNSUtils<SolutionType = WeeklySolution>,
    WeeklySolution: Solution,
    WeeklyParameters: Parameters,
    Ss: SystemSolutions<Weekly = WeeklySolution>,
{
    type Algorithm = Algorithm<
        WeeklySolution,
        WeeklyParameters,
        PriorityQueue<WorkOrderNumber, i64>,
        WeeklyOptions,
        Ss,
    >;
    type Options = WeeklyOptions;

    /// Incorporates the system solution by updating internal state and force
    /// scheduling work orders
    fn incorporate_system_solution(&mut self) -> Result<bool>
    {
        let mut state_change = true;
        let periods = self.parameters.weekly_periods.clone();
        // Weekly loops over all parameters
        for (work_order_number, weekly_parameter) in
            self.parameters.weekly_work_order_parameters.clone().iter()
        {
            // Project model takes precedence over the weekly
            let project_scheduled_period = self
                .loaded_system_solution
                .project_actor_solution()
                .ok()
                .and_then(|project_solution| {
                    project_solution.project_period(work_order_number, &periods)
                });

            let weekly_scheduled_period = self
                .solution
                .weekly_scheduled_work_orders
                .get_mut(work_order_number)
                .with_context(|| {
                    format!("{work_order_number:?}\nis not found in the WeeklyAlgorithm")
                })?;

            // If the [`ProjectAlgorithm`] have the [`WorkOrder`] the [`Weekly`]
            // should respect this, but not schedule it and use resources.
            if let Some(project_period) = project_scheduled_period {
                if *weekly_scheduled_period != WhereIsWorkOrder::Project(project_period.clone()) {
                    state_change = true
                };
                *weekly_scheduled_period = WhereIsWorkOrder::Project(project_period.clone())
            }

            if weekly_parameter.locked_in_period == weekly_scheduled_period.clone() {
                continue;
            }
            state_change = true;
        }

        // CRUCIAL: Forced work orders are always part of the shared state in
        // incorporate() This function should determine what is forced and only
        // update WeeklyParameters

        self.force_schedule()?;
        Ok(state_change)
    }

    fn make_atomic_pointer_swap(&mut self)
    {
        // TODO: Optimize by using Cow or selectively cloning only needed fields
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            // Swap the weekly solution in shared state
            shared_solution.weekly_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<<<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective>,
    >
    {
        let mut new_objective_value = WeeklyObjectiveValue::new(&self.options);

        self.determine_urgency(&mut new_objective_value)
            .context("could not determine weekly urgency")?;

        self.determine_resource_penalty(&mut new_objective_value);

        self.determine_clustering(&mut new_objective_value)
            .context("Could not determine WeeklyObjective value")?;

        self.determine_percent_scheduled(&mut new_objective_value)?;

        new_objective_value.aggregate_objectives();

        // We always work on self and substitute remaining parts
        if new_objective_value.objective_value < self.solution.objective_value().objective_value {
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            Ok(ObjectiveValueType::Worse(new_objective_value))
        }
    }

    fn schedule(&mut self) -> Result<()>
    {
        // TODO: Refactor to separate schedule and shared_state_update concerns

        while !self.solution_intermediate.is_empty() {
            for period in self.parameters.weekly_periods.clone() {
                let (work_order_number, weight) = match self.solution_intermediate.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),

                    None => {
                        break;
                    }
                };

                // TODO: Review overloaded logic here
                let inf_work_order_number = self
                    .schedule_weekly_work_order(work_order_number, &period)
                    .with_context(|| {
                        format!("{work_order_number:?} could not be scheduled normally")
                    })?;

                if let Some(work_order_number) = inf_work_order_number {
                    if &period != self.parameters.weekly_periods.last().unwrap() {
                        self.solution_intermediate.push(work_order_number, weight);
                    }
                } else {
                    self.assert_work_load_to_loading(work_order_number, &period)
                        .unwrap()
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self) -> Result<()>
    {
        let mut rng = rand::rng();
        let weekly_work_orders = self.solution.every_work_order();

        let weekly_parameters = &self.parameters.weekly_work_order_parameters;

        let mut filtered_keys: Vec<_> = weekly_work_orders
            .iter()
            .filter(|(won, _)| {
                weekly_parameters
                    .get(won)
                    .unwrap()
                    .locked_in_period
                    .not_scheduled()
            })
            .map(|(&won, _)| won)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(&mut rng, self.options.number_of_removed_work_orders)
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_keys {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule: {work_order_number:?}\nLocation: {}",
                        Location::caller()
                    )
                })?;

            let weight = self
                .parameters
                .weekly_work_order_parameters
                .get(work_order_number)
                .context("Parameters should always be available")?
                .weight;
            self.solution_intermediate.push(*work_order_number, weight);
        }
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm
    {
        &mut self.0
    }

    #[allow(unreachable_code, unused_variables)]
    fn force_schedule(&mut self) -> Result<()>
    {
        // ISSUE #999 - Disabled temporarily
        return Ok(());

        // TODO: Derive WeeklyParameter::locked_in_period from SchedulingEnvironment
        let forced_work_orders: Vec<_> = self
            .parameters
            .weekly_work_order_parameters
            .iter()
            .filter(|e| e.1.locked_in_period.weekly_forced())
            .map(|e| {
                // TODO: Remove ForcedWorkOrder if unnecessary
                ForcedWorkOrder::Locked(*e.0)
            })
            .collect();

        for forced_work_order_numbers in &forced_work_orders {
            self.schedule_forced_weekly_work_order(forced_work_order_numbers)
                .with_context(|| {
                    format!("{forced_work_order_numbers:#?} could not be force scheduled")
                })?;
        }

        Ok(())
    }

    fn throttling(&self, throttling: &ordinator_configuration::throttling::Throttling) -> u64
    {
        throttling.weekly_throttling
    }
}

impl<Ss> WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // pub fn swap_scheduled_work_orders(&mut self, rng: &mut impl rand::Rng) {
    //         let scheduled_work_orders: Vec<_> = self
    //             .weekly_solution
    //             .weekly_periods
    //             .keys()
    //             .cloned()
    //             .collect();
    //         let randomly_chosen = scheduled_work_orders.choose_multiple(rng,
    // 2).collect::<Vec<_>>();         unsafe {
    //             let scheduled_work_order_1 =
    // self.solution.weekly_periods.get_mut(randomly_chosen[0]).unwrap() as *mut
    // Option<Period>;             let scheduled_work_order_2 =
    // self.solution.weekly_periods.get_mut(randomly_chosen[1]).unwrap() as *mut
    // Option<Period>;             std::ptr::swap(scheduled_work_order_1,
    // scheduled_work_order_2);             // You cannot do this anymore
    // either. What is the best remedy for this?
    // self.update_loadings(randomly_chosen[0],
    // (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[1],
    // (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[0],
    // (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Add);
    //             self.update_loadings(randomly_chosen[1],
    // (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Add);
    //         }
    // }

    fn weekly_capacity_by_resource(&self, resource: &Skill, period: &Period) -> Result<Work>
    {
        self.parameters
            .weekly_capacity
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    fn weekly_loading_by_resource(&self, resource: &Skill, period: &Period) -> Result<Work>
    {
        self.solution
            .weekly_loadings
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    #[allow(dead_code)]
    pub fn calculate_utilization(&self) -> Result<Vec<(i64, u64)>>
    {
        let mut utilization_by_period = Vec::new();

        for (index, period) in self.parameters.weekly_periods.iter().enumerate() {
            let mut intermediate_loading: f64 = 0.0;
            let mut intermediate_capacity: f64 = 0.0;
            for resource in Skill::iter() {
                let loading = self.weekly_loading_by_resource(&resource, period)?;
                let capacity = self.weekly_capacity_by_resource(&resource, period)?;

                intermediate_loading += loading.to_f64();
                intermediate_capacity += capacity.to_f64();
            }
            let percentage_loading =
                ((intermediate_loading / intermediate_capacity) * 100.0) as u64;
            utilization_by_period.push((index as i64, percentage_loading));
        }
        Ok(utilization_by_period)
    }

    fn determine_urgency(&mut self, weekly_objective_value: &mut WeeklyObjectiveValue)
    -> Result<()>
    {
        for (work_order_number, scheduled_period) in self.solution.every_work_order() {
            let optimized_period = match scheduled_period {
                WhereIsWorkOrder::Weekly(optimized_period) => optimized_period,
                // CRUCIAL: Objective should be based on Project solution
                WhereIsWorkOrder::Project(period) => period,

                WhereIsWorkOrder::NotScheduled => self
                    .parameters
                    .weekly_periods
                    .last()
                    .context("There should always be a last .parameters.eriod")?,
            };

            let work_order_latest_allowed_finish_period = &self
                .parameters
                .weekly_work_order_parameters
                .get(work_order_number)
                .expect("WeeklyParameter should always be available for the WeeklySolution")
                .latest_period;

            let non_zero_period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );

            let work_order_value = self
                .parameters
                .weekly_work_order_parameters
                .get(work_order_number)
                .unwrap()
                .weight;

            let period_penalty = non_zero_period_difference * work_order_value;

            weekly_objective_value.urgency.1.checked_add_assign(&period_penalty)
                .ok()
                .with_context(|| format!("Overflow on the weekly urgency.\nperiod penalty: {period_penalty}\nperiod difference: {non_zero_period_difference}\nwork_order_value: {work_order_value}"))?;
        }
        Ok(())
    }

    fn determine_clustering(
        &mut self,
        weekly_objective_value: &mut WeeklyObjectiveValue,
    ) -> anyhow::Result<()>
    {
        for period in &self.parameters.weekly_periods {
            // Precompute scheduled work orders for the current period
            let scheduled_work_orders_by_period: Vec<_> = self
                .solution
                .every_work_order()
                .iter()
                .filter_map(|(won, where_is_period)| match where_is_period {
                    WhereIsWorkOrder::Weekly(opt_per) => {
                        if opt_per == period {
                            Some(won)
                        } else {
                            None
                        }
                    }
                    WhereIsWorkOrder::Project(opt_per) => {
                        if opt_per == period {
                            Some(won)
                        } else {
                            None
                        }
                    }
                    WhereIsWorkOrder::NotScheduled => None,
                })
                .collect();

            // Cache references to clustering inner map
            let clustering_inner = &self.parameters.weekly_clustering.inner;

            for i in 0..scheduled_work_orders_by_period.len() {
                for j in (i + 1)..scheduled_work_orders_by_period.len() {
                    // Retrieve clustering value, handling symmetry
                    let work_order_pair = (
                        *scheduled_work_orders_by_period[i],
                        *scheduled_work_orders_by_period[j],
                    );
                    let reverse_pair = (
                        *scheduled_work_orders_by_period[j],
                        *scheduled_work_orders_by_period[i],
                    );

                    let clustering_value_for_work_order_pair = clustering_inner
                        .get(&work_order_pair)
                        .or_else(|| clustering_inner.get(&reverse_pair))
                        .with_context(|| {
                            format!(
                                "Missing: {} between {:?} and {:?}",
                                std::any::type_name::<WeeklyClustering>(),
                                scheduled_work_orders_by_period[i],
                                scheduled_work_orders_by_period[j]
                            )
                        })
                        .context("clustering_value not available. Did you disable it to increase startup times? That is the most likely scenario")?;

                    // Increment the clustering value in the objective
                    weekly_objective_value.clustering_value.1 +=
                        *clustering_value_for_work_order_pair;
                }
            }
        }
        Ok(())
    }

    // Calculate resource penalty based on total exceeded hours
    fn determine_resource_penalty(&mut self, weekly_objective_value: &mut WeeklyObjectiveValue)
    {
        for (period, skill_map) in &self.parameters.weekly_capacity.0 {
            let capacity: f64 = skill_map.values().map(|w| w.to_f64()).sum();
            let loading: f64 = self
                .solution
                .weekly_loadings
                .0
                .get(period)
                .unwrap()
                .values()
                .map(|w| w.to_f64())
                .sum();

            if loading - capacity > 0.0 {
                weekly_objective_value.resource_penalty.1 += (loading - capacity) as i64
            }
        }
    }

    fn assert_work_load_to_loading(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<()>
    {
        let work_order_parameter = self
            .parameters
            .weekly_work_order_parameters
            .get(&work_order_number)
            .unwrap();
        let work_load = &work_order_parameter.work_load;
        let locked_in_period = &work_order_parameter.locked_in_period;
        let weekly_loadings = self.solution.weekly_loadings.0.get(period).unwrap().clone();
        let weekly_capacity = self
            .parameters
            .weekly_capacity
            .0
            .get(period)
            .unwrap()
            .clone();
        ensure!(
            combined_loadings(work_load, &weekly_loadings)
                .iter()
                .all(|(res, work)| work >= work_load.get(res).unwrap()),
            "The amount of work loaded into the schedule and the work_load of the work order does not match.\n\
            possible errors:\n\
            * Rounding error\n\
            * Calculation error\n\
            * Timing error in either pointer swaps or user-input message\n\
            combined_loadings: {:#?}\n\
            combined_work_load: {:#?}\n\
            combined_capacities: {:#?}\n\
            work_load: {:#?}\n\
            locked_in_period: {:#?}\n\
            period: {:#?}\n\
            length of priority queue: {:#?}\n\
            Location: {}",
            combined_loadings(work_load, &weekly_loadings),
            work_load.clone().into_values().sum::<Work>(),
            combined_loadings(work_load, &weekly_capacity),
            work_load,
            locked_in_period,
            period,
            self.solution_intermediate.len(),
            Location::caller(),
        );
        Ok(())
    }

    fn determine_percent_scheduled(
        &self,
        new_objective_value: &mut WeeklyObjectiveValue,
    ) -> Result<()>
    {
        let total_work_orders = self.parameters.weekly_work_order_parameters.len() as u64;
        let scheduled_work_orders = self
            .solution
            .weekly_scheduled_work_orders
            .iter()
            .filter(|(_, v)| match v {
                WhereIsWorkOrder::Weekly(_period) => true,
                WhereIsWorkOrder::Project(_) => false,
                WhereIsWorkOrder::NotScheduled => false,
            })
            .count() as u64;

        new_objective_value.percent_scheduled.1 =
            Percent::new(scheduled_work_orders, total_work_orders)
                .context("percent scheduled could not be calculated")?;
        Ok(())
    }
}

// TODO: Consolidate with other models
#[derive(Debug)]
pub enum ForcedWorkOrder
{
    Locked(WorkOrderNumber),
    FromProject((WorkOrderNumber, Period)),
}

impl ForcedWorkOrder
{
    pub fn work_order_number(&self) -> &WorkOrderNumber
    {
        match self {
            ForcedWorkOrder::Locked(work_order_number) => work_order_number,
            ForcedWorkOrder::FromProject((work_order_number, _)) => work_order_number,
        }
    }
}

#[derive(Debug)]
pub enum ScheduleWorkOrder
{
    Normal,
    Forced,
    Unschedule,
}

// TODO: Move this trait to ProjectSolution interface (defines "Metavariables"
// from the paper)
pub trait WeeklyUtils
{
    fn schedule_weekly_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>>;

    fn schedule_forced_weekly_work_order(
        &mut self,
        force_schedule_work_order: &ForcedWorkOrder,
    ) -> Result<()>;

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool;

    /// This function updates the WeeklyResources based on the a provided
    /// loading.
    fn update_loadings(&mut self, weekly_resources: WeeklyResources, load_operation: LoadOperation);
}
// TODO: Use binary heap instead
impl<Ss> WeeklyUtils for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // TODO: Rely on interface instead. This function should determine period for
    // project work order's first day

    fn schedule_weekly_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>>
    {
        let weekly_parameter = self
            .parameters
            .weekly_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .clone();

        let work_load: &HashMap<_, _> = &weekly_parameter
            .work_load
            .iter()
            .map(|e| (*e.0, e.1.round()))
            .collect();

        warn!(target: "stdout", work_order_number = %work_order_number, period = %period, parameters = %self.parameters.state());
        if weekly_parameter.excluded_periods.contains(period) {
            return Ok(Some(work_order_number));
        }

        warn!(target: "stdout", work_order_number = %work_order_number, period = %period);
        if self.parameters.period_locks.contains(period) {
            return Ok(Some(work_order_number));
        }
        warn!(target: "stdout", work_order_number = %work_order_number, period = %period);

        // If no `WeeklyResources` could be determined for the `schedule` decision make
        // an early return.
        //
        // TODO [ ] - replace with multi-skill calculation. You do not need
        // STARTHERE
        let previous_period = self
            .solution
            .set_work_order_to_weekly(work_order_number, period.clone());

        ensure!(
            previous_period.as_ref().unwrap().not_scheduled(),
            "Previous period: {:#?}\nNew period: {:#?}\nWeeklyParameter: {:#?}\nfile: {}\nline: {}",
            &previous_period,
            period,
            weekly_parameter,
            file!(),
            line!()
        );

        let weekly_resources =
            WeeklyResources(HashMap::from([(period.clone(), work_load.clone())]));
        self.update_loadings(weekly_resources, LoadOperation::Add);
        self.assert_work_load_to_loading(work_order_number, period)
            .with_context(|| {
                format!(
                    "Calculated work_load: {:#?}\nLocation: {}",
                    work_load,
                    Location::caller()
                )
            })?;

        Ok(None)
    }

    // Ensures forced work orders are scheduled in correct order using template
    // trait pattern
    fn schedule_forced_weekly_work_order(
        &mut self,
        force_schedule_work_order: &ForcedWorkOrder,
    ) -> Result<()>
    {
        if self.is_scheduled(force_schedule_work_order.work_order_number()) {
            self.unschedule_specific_work_order(*force_schedule_work_order.work_order_number())
                .with_context(|| {
                    format!(
                        "{:#?}\nfile: {}\nline: {}",
                        force_schedule_work_order,
                        file!(),
                        line!()
                    )
                })?;
        }

        // Forced work orders come from SchedulingEnvironment state
        let locked_in_period = match &force_schedule_work_order {
            ForcedWorkOrder::Locked(work_order_number) => self
                .parameters
                .get_locked_in_period(work_order_number)
                .clone(),
            // If Project scheduled the work order, Weekly should not reschedule it
            ForcedWorkOrder::FromProject((_, period)) => period.clone(),
        };

        // TODO: Move update loadings logic to SchedulingEnvironment

        let work_order_number = force_schedule_work_order.work_order_number();

        self.solution
            // TODO: Move this interface to higher system level
            .set_work_order_to_weekly(*work_order_number, locked_in_period.clone())
            .with_context(|| {
                format!(
                    "Could not fully update {:#?} in {}",
                    force_schedule_work_order, &locked_in_period
                )
            })?;

        let work_load = self
            .parameters
            .weekly_work_order_parameters
            .get(force_schedule_work_order.work_order_number())
            .unwrap()
            .work_load
            .clone();

        let weekly_resources =
            WeeklyResources(HashMap::from([(locked_in_period, work_load.clone())]));
        weekly_resources.assert_well_shaped_resources()?;
        weekly_resources.assert_well_shaped_resources()?;

        self.update_loadings(weekly_resources, LoadOperation::Add);
        Ok(())
    }

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool
    {
        self.solution
            .every_work_order()
            .get(work_order_number)
            .expect("This should always be initialized")
            .weekly_forced()
    }

    /// Updates WeeklyResources based on the provided loading
    /// The weekly loadings should not be dependent on the skill, we only
    /// handle it in the objective function and as a predicate in the
    /// `schedule` method.
    fn update_loadings(
        &mut self,
        // This should simply be the "work load"
        weekly_resources: WeeklyResources,
        load_operation: LoadOperation,
    )
    {
        // TODO: Refactor to handle changes correctly without permutation loop
        //
        // This should look exactly like the... You simply have to add the loadings. You
        // cannot make this difficult to understand.
        for (period, work_load) in weekly_resources.0 {
            for (skill, work) in work_load {
                match load_operation {
                    LoadOperation::Add => {
                        self.solution
                            .weekly_loadings
                            .0
                            .get_mut(&period)
                            .expect("All Periods should be initialized at this point")
                            // What happens if the value is not present? Should we simply insert it?
                            .entry(skill)
                            .and_modify(|e| {
                                *e += work;
                            })
                            .or_insert(work);
                    }
                    LoadOperation::Sub => {
                        let weekly_loading = self
                            .solution
                            .weekly_loadings
                            .0
                            .get_mut(&period)
                            .expect("All Periods should be initialized at this point")
                            .get_mut(&skill)
                            .unwrap();

                        *weekly_loading -= work;
                    }
                }
            }
        }
    }
}

/// Combines skill-based loadings for a work order
fn combined_loadings(
    work_load: &HashMap<Skill, Work>,
    weekly_loading_resources: &HashMap<Skill, Work>,
) -> HashMap<Skill, Work>
{
    weekly_loading_resources
        .iter()
        .filter(|(skill, _)| work_load.contains_key(skill))
        .map(|(skill, work)| (*skill, *work))
        .collect()
}

#[allow(dead_code)]
fn assert_work_load_equal_to_weekly_resource(
    period: &Period,
    weekly_resource_loadings: &WeeklyResources,
    work_load: &HashMap<Skill, Work>,
    load_operation: LoadOperation,
) -> Result<()>
{
    let aggregate_weekly_resource = weekly_resource_loadings
            .0
            .get(period)
            .with_context(|| format!("{:#?}\nnot present. This probably means that nothing was {:#?}\nfile: {}\nline: {}", period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
            .values()
            .fold(Work::from(0.0), |acc, w| acc + *w);

    let aggregate_work_load =
        work_load
            .values()
            .fold(Work::from(0.0), |acc, wor| match load_operation {
                LoadOperation::Add => acc + *wor,
                LoadOperation::Sub => acc - *wor,
            });

    let value = aggregate_work_load.equal(aggregate_weekly_resource);

    ensure!(
        value,
        format!(
            "Aggregate Work:\nWeeklyResources: {:#?}\nwork_load: {:#?}\n\n{:#?} {:#?}\nfile: {}\nline: {}",
            aggregate_weekly_resource,
            aggregate_work_load,
            work_load,
            weekly_resource_loadings,
            file!(),
            line!()
        )
    );
    Ok(())
}

pub fn calculate_period_difference(scheduled_period: &Period, latest_period: &Period) -> i64
{
    let scheduled_period_date = scheduled_period.finish_datetime().to_owned();
    let latest_date = latest_period.finish_datetime();
    let duration = scheduled_period_date.signed_duration_since(latest_date);
    let days = duration.num_days();
    std::cmp::max(days / 7, 0) as i64
}

impl<Ss> WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // ISSUE #000 - Pending implementation
    pub fn update_resources_state(
        &mut self,
        _weekly_resources_request: WeeklyRequestResource,
    ) -> Result<WeeklyResponseResources>
    {
        // TODO: Implement resource state update logic
        // match weekly_resources_request {

        //     WeeklyRequestResource::GetLoadings {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let loading = &self.solution.weekly_loadings;

        //         let weekly_response_resources =
        //
        // WeeklyResponseResources::LoadingAndCapacities(loading.clone());
        //         Ok(weekly_response_resources)
        //     }
        //     WeeklyRequestResource::GetCapacities {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let capacities = &self.parameters.weekly_capacity;

        //         let weekly_response_resources =
        //
        // WeeklyResponseResources::LoadingAndCapacities(capacities.clone());
        //         Ok(weekly_response_resources)
        //     }
        //     WeeklyRequestResource::GetPercentageLoadings {
        //         periods_end: _,
        //         resources: _,
        //     } => {
        //         let capacities = &self.parameters.weekly_capacity;
        //         let loadings = &self.solution.weekly_loadings;

        //         Algorithm::assert_that_capacity_is_respected(loadings, capacities)
        //             .context("Loadings exceed the capacities")?;
        //         Ok(WeeklyResponseResources::Percentage(
        //             capacities.clone(),
        //             loadings.clone(),
        //         ))
        //     }
        // }
        todo!()
    }

    #[instrument(level = "info", skip_all)]
    pub fn update_scheduling_state(
        &mut self,
        weekly_scheduling_request: WeeklyRequestScheduling,
    ) -> Result<WeeklyResponseScheduling>
    {
        match weekly_scheduling_request {
            WeeklyRequestScheduling::Schedule(schedule_work_order) => {
                let period = self
                    .parameters
                    .weekly_periods
                    .iter()
                    .find(|period| period.period_string() == schedule_work_order.period_string())
                    .cloned()
                    .with_context(|| {
                        format!(
                            "period: {:?} does not exist",
                            schedule_work_order.period_string()
                        )
                    })?;

                let mut number_of_work_orders = 0;
                for work_order_number in schedule_work_order.work_order_number {
                    let weekly_parameter = self
                        .parameters
                        .weekly_work_order_parameters
                        .get_mut(&work_order_number)
                        .unwrap();
                    if weekly_parameter.excluded_periods.contains(&period) {
                        weekly_parameter.excluded_periods.remove(&period);
                    }
                    self.parameters
                        .set_locked_in_period(work_order_number, period.clone())
                        .context("could not set locked in period")?;
                    number_of_work_orders += 1;
                }

                Ok(WeeklyResponseScheduling::new(number_of_work_orders, period))
            }
            WeeklyRequestScheduling::ExcludeFromPeriod(_exclude_from_period) => {
                todo!(
                    "We should never hit this point. All logic mutating the `Parameters` have
                    moved on the `StateLink` handler`"
                );
            }
        }
    }

    fn unschedule_specific_work_order(&mut self, work_order_number: WorkOrderNumber) -> Result<()>
    {
        let unschedule_from_period = self
            .solution
            .set_work_order_to_unschedule(work_order_number)
            .context("WorkOrder unschedule should never be called on a not scheduled WorkOrder")?;

        if let WhereIsWorkOrder::Weekly(unschedule_from_period) = unschedule_from_period {
            let weekly_parameter = self
                .parameters
                .weekly_work_order_parameters
                .get(&work_order_number)
                .unwrap();

            let work_load = weekly_parameter.work_load.clone();

            // let weekly_resources = self
            //     .determine_best_permutation(work_load, &unschedule_from_period,
            // ScheduleWorkOrder::Unschedule)     .with_context(||
            // format!("{:#?}\n{:#?}\nfor {:?}\nfile: {}\nline: {}", weekly_parameter,
            // unschedule_from_period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
            //     .context("Determining the WeeklyResources associated with a unscheduling
            // operation should always be possible")?;
            //

            let weekly_resources =
                WeeklyResources(HashMap::from([(unschedule_from_period, work_load.clone())]));
            weekly_resources.assert_well_shaped_resources()?;
            self.update_loadings(weekly_resources, LoadOperation::Sub);
        }
        Ok(())
    }

    // TODO: Refactor populate_priority_queue
    pub fn populate_priority_queue(&mut self)
    {
        for work_order_number in self.solution.every_work_order().clone().keys() {
            let weekly_parameter = self
                .parameters
                .weekly_work_order_parameters
                .get(work_order_number)
                .expect("The WeeklyParameter should always be available for the WeeklySolution");

            if weekly_parameter.locked_in_period.weekly_forced() {
                continue;
            }

            if self
                .solution
                .every_work_order()
                .get(work_order_number)
                .unwrap()
                .not_scheduled()
            {
                let weekly_work_order_weight = weekly_parameter.weight;
                self.solution_intermediate
                    .push(*work_order_number, weekly_work_order_weight);
            }
        }
    }
}

impl<Ss: SystemSolutions + fmt::Debug> Inspect for WeeklyAlgorithm<Ss>
{
    fn summary(&self) -> impl fmt::Display + '_
    {
        struct Summary<'a>
        {
            id: &'a str,
            stagnation: u64,
            version: u64,
            objective: i64,
        }
        impl fmt::Display for Summary<'_>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
            {
                write!(
                    f,
                    "WeeklyAlgorithm {}: v{}, stagnation {}, objective {}",
                    self.id, self.version, self.stagnation, self.objective
                )
            }
        }
        let (stagnation, version) = self.0.solution.stagnation_and_version();
        Summary {
            id: &self.0.id.0,
            stagnation,
            version,
            objective: self.0.solution.objective_value().objective_value,
        }
    }

    fn state(&self) -> impl fmt::Display + '_
    {
        struct State<'a, P: Inspect>
        {
            id: &'a str,
            stagnation: u64,
            version: u64,
            objective: &'a dyn fmt::Debug,
            scheduled: usize,
            total: usize,
            parameters: &'a P,
        }
        impl<P: Inspect> fmt::Display for State<'_, P>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
            {
                writeln!(f, "WeeklyAlgorithm {}:", self.id)?;
                writeln!(
                    f,
                    "  version: {}, stagnation: {}",
                    self.version, self.stagnation
                )?;
                writeln!(f, "  objective: {:?}", self.objective)?;
                writeln!(f, "  scheduled: {}/{}", self.scheduled, self.total)?;
                write!(f, "  parameters: {}", self.parameters.summary())
            }
        }
        let (stagnation, version) = self.0.solution.stagnation_and_version();
        let scheduled = self
            .0
            .solution
            .weekly_scheduled_work_orders
            .values()
            .filter(|w| !w.not_scheduled())
            .count();
        let total = self.0.solution.weekly_scheduled_work_orders.len();
        State {
            id: &self.0.id.0,
            stagnation,
            version,
            objective: self.0.solution.objective_value(),
            scheduled,
            total,
            parameters: &self.0.parameters,
        }
    }
}

impl<Ss>
    From<
        Algorithm<
            WeeklySolution,
            WeeklyParameters,
            PriorityQueue<WorkOrderNumber, i64>,
            WeeklyOptions,
            Ss,
        >,
    > for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(
        value: Algorithm<
            WeeklySolution,
            WeeklyParameters,
            PriorityQueue<WorkOrderNumber, i64>,
            WeeklyOptions,
            Ss,
        >,
    ) -> Self
    {
        WeeklyAlgorithm(value)
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use weekly_parameters::WeeklyWorkOrderParameter;

    use super::*;

    impl WeeklyWorkOrderParameter
    {
        pub fn new(
            locked_in_period: WhereIsWorkOrder<Period>,
            excluded_periods: HashSet<Period>,
            latest_period: Period,
            weight: i64,
            work_load: HashMap<Skill, Work>,
        ) -> Self
        {
            Self {
                locked_in_period,
                excluded_periods,
                latest_period,
                weight,
                work_load,
            }
        }
    }

    #[test]
    fn test_update_load_1()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::MtnMech;
        let load = Work::from(30.0);

        let weekly_resources_inner = HashMap::from([(
            period.clone(),
            HashMap::from([
                (Skill::MtnMech, Work::from(100.0)),
                (Skill::MtnElec, Work::from(100.0)),
                (Skill::Prodtech, Work::from(100.0)),
            ]),
        )]);

        let mut weekly_resources = WeeklyResources::new(weekly_resources_inner);

        weekly_resources.update_load(&period, resource, load, LoadOperation::Add);

        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::MtnMech)
                .unwrap(),
            Work::from(130.0)
        );
        // Other skills should be unchanged
        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::MtnElec)
                .unwrap(),
            Work::from(100.0)
        );
    }

    #[test]
    fn test_update_load_2()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::VenMech;
        let load = Work::from(30.0);

        let weekly_resources_inner = HashMap::from([(
            period.clone(),
            HashMap::from([
                (Skill::MtnMech, Work::from(100.0)),
                (Skill::MtnElec, Work::from(100.0)),
                (Skill::Prodtech, Work::from(100.0)),
            ]),
        )]);

        let mut weekly_resources = WeeklyResources::new(weekly_resources_inner);

        weekly_resources.update_load(&period, resource, load, LoadOperation::Add);

        // New skill should be inserted with the load value
        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::VenMech)
                .unwrap(),
            Work::from(30.0)
        );
        // Existing skills should be unchanged
        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::MtnMech)
                .unwrap(),
            Work::from(100.0)
        );
    }

    #[test]
    fn test_update_load_3()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::MtnMech;
        let load = Work::from(30.0);

        let weekly_resources_inner = HashMap::from([(
            period.clone(),
            HashMap::from([
                (Skill::MtnMech, Work::from(100.0)),
                (Skill::MtnElec, Work::from(100.0)),
                (Skill::Prodtech, Work::from(100.0)),
            ]),
        )]);

        let mut weekly_resources = WeeklyResources::new(weekly_resources_inner);

        weekly_resources.update_load(&period, resource, load, LoadOperation::Sub);

        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::MtnMech)
                .unwrap(),
            Work::from(70.0)
        );
        // Other skills should be unchanged
        assert_eq!(
            *weekly_resources
                .0
                .get(&period)
                .unwrap()
                .get(&Skill::MtnElec)
                .unwrap(),
            Work::from(100.0)
        );
    }

    // Tests for the removed per-operator permutation functions have been removed
    // since determine_best_permutation now works with aggregate HashMap<Skill,
    // Work> data.

    // Should this test go into the integration testing instead? I
    // think that is a really good idea. Also you should never s
    #[test]
    fn test_unschedule_random_work_orders() -> Result<()>
    {
        let periods: Vec<Period> = vec![
            Period::from_str("2023-W47-48").unwrap(),
            Period::from_str("2023-W49-50").unwrap(),
        ];

        let _latest_period = Period::from_str("2023-W49-50").unwrap();

        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Skill::MtnMech, Work::from(10.0));
        work_load_1.insert(Skill::MtnElec, Work::from(10.0));
        work_load_1.insert(Skill::Prodtech, Work::from(10.0));

        work_load_2.insert(Skill::MtnMech, Work::from(20.0));
        work_load_2.insert(Skill::MtnElec, Work::from(20.0));
        work_load_2.insert(Skill::Prodtech, Work::from(20.0));

        work_load_3.insert(Skill::MtnMech, Work::from(30.0));
        work_load_3.insert(Skill::MtnElec, Work::from(30.0));
        work_load_3.insert(Skill::Prodtech, Work::from(30.0));

        let mut weekly_resources = WeeklyResources::default();

        weekly_resources.insert_skill_work(periods[0].clone(), Skill::MtnMech, Work::from(40.0));
        weekly_resources.insert_skill_work(periods[0].clone(), Skill::MtnElec, Work::from(40.0));
        weekly_resources.insert_skill_work(periods[1].clone(), Skill::MtnMech, Work::from(40.0));
        weekly_resources.insert_skill_work(periods[1].clone(), Skill::MtnElec, Work::from(40.0));

        // This way of making parameters needs to go away. Is the right call here to
        // simply delete the let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let mut weekly_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let weekly_parameter_1 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_1,
        // );

        // let weekly_parameter_2 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_2,
        // );

        // let weekly_parameter_3 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_3,
        // );

        // let work_order_number_1 = WorkOrderNumber(2200000001);
        // let work_order_number_2 = WorkOrderNumber(2200000002);
        // let work_order_number_3 = WorkOrderNumber(2200000003);

        // weekly_parameters
        //     .weekly_work_order_parameters
        //     .insert(work_order_number_1, weekly_parameter_1);

        // weekly_parameters
        //     .weekly_work_order_parameters
        //     .insert(work_order_number_2, weekly_parameter_2);

        // weekly_parameters
        //     .weekly_work_order_parameters
        //     .insert(work_order_number_3, weekly_parameter_3);

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let weekly_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let weekly_solution = WeeklySolution::new(&weekly_parameters);

        // Actor::builder().agent_id(&id).
        // scheduling_environment(scheduling_environment).algorithm(|con|con.id(&id).
        // parameters(options, scheduling_environment)).configurations();

        // let mut weekly_algorithm = Algorithm::new(
        //     &Id::default(),
        //     weekly_solution,
        //     weekly_parameters,
        //     ArcSwapSharedSolution::default().into(),
        // );

        // weekly_algorithm
        //     .solution
        //     .weekly_scheduled_work_orders
        //     .insert(work_order_number_1, Some(periods[0].clone()));
        // weekly_algorithm
        //     .solution
        //     .weekly_scheduled_work_orders
        //     .insert(work_order_number_2, Some(periods[1].clone()));
        // weekly_algorithm
        //     .solution
        //     .weekly_scheduled_work_orders
        //     .insert(work_order_number_3, Some(periods[1].clone()));

        // let operational_resource_0 = OperationalResource::new("OP_TEST_0",
        // Work::from(30.0), vec![     Resources::MtnMech,
        //     Resources::MtnElec,
        //     Resources::Prodtech,
        // ]);
        // let operational_resource_1 =
        //     OperationalResource::new("OP_TEST_1", Work::from(150.0), vec![
        //         Resources::MtnMech,
        //         Resources::MtnElec,
        //         Resources::Prodtech,
        //     ]);

        // weekly_algorithm
        //     .solution
        //     .weekly_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);
        // weekly_algorithm
        //     .solution
        //     .weekly_loadings
        //     .insert_operational_resource(periods[1].clone(), operational_resource_1);

        // let seed: [u8; 32] = [
        //     1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        // 21, 22, 23, 24,     25, 26, 27, 28, 29, 30, 31, 32,
        // ];

        // let rng = StdRng::from_seed(seed);

        // let weekly_options = WeeklyOptions {
        //     number_of_removed_work_order: 2,
        //     rng,
        //     urgency_weight: 1,
        //     resource_penalty_weight: 1,
        //     clustering_weight: 1,
        //     work_order_configurations: todo!(),
        //     material_to_period: todo!(),
        // };

        // weekly_algorithm.options = weekly_options;

        // weekly_algorithm.unschedule().expect(
        //     "It should always be possible to unschedule random work orders in the
        // weekly agent", );

        // assert_eq!(
        //     *weekly_algorithm
        //         .solution
        //         .weekly_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000001))
        //         .unwrap(),
        //     Some(Period::from_str("2023-W47-48").unwrap())
        // );

        // assert_eq!(
        //     *weekly_algorithm
        //         .solution
        //         .weekly_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000002))
        //         .unwrap(),
        //     None
        // );

        // assert_eq!(
        //     *weekly_algorithm
        //         .solution
        //         .weekly_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000003))
        //         .unwrap(),
        //     None
        // );
        Ok(())
    }

    #[test]
    fn test_calculate_period_difference_1()
    {
        let scheduled_period = Period::from_str("2023-W47-48");
        let latest_period = Period::from_str("2023-W49-50");

        let difference =
            calculate_period_difference(&scheduled_period.unwrap(), &latest_period.unwrap());

        assert_eq!(difference, 0);
    }
    #[test]
    fn test_calculate_period_difference_2()
    {
        let period_1 = Period::from_str("2023-W47-48");
        let period_2 = Period::from_str("2023-W45-46");

        let difference = calculate_period_difference(&period_1.unwrap(), &period_2.unwrap());

        assert_eq!(difference, 2);
    }

    #[test]
    fn test_choose_multiple()
    {
        for _ in 0..19 {
            let seed: [u8; 32] = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            let mut rng = StdRng::from_seed(seed);

            assert_eq!(
                [1, 2, 3].choose_multiple(&mut rng, 2).collect::<Vec<_>>(),
                [&3, &2]
            );
        }
    }

    #[test]
    fn test_unschedule_work_order_none_in_scheduled_period() -> Result<()>
    {
        let _work_order_number = WorkOrderNumber(2100000001);
        let periods = [Period::from_str("2026-W41-42").unwrap()];
        let mut weekly_resources = WeeklyResources::default();

        weekly_resources.insert_skill_work(periods[0].clone(), Skill::MtnMech, Work::from(40.0));
        weekly_resources.insert_skill_work(periods[0].clone(), Skill::MtnElec, Work::from(40.0));

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let mut weekly_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let weekly_parameter = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     periods[0].clone(),
        //     1000,
        //     HashMap::from([(Resources::MtnMech, Work::from(5.0))]),
        // );

        // weekly_parameters
        //     .weekly_work_order_parameters
        //     .insert(work_order_number, weekly_parameter);

        // let project_solution_builder = ProjectSolutionBuilder::new();

        // let mut project_days = HashMap::new();
        // project_days.insert(work_order_number, WhereIsWorkOrder::NotScheduled);

        // let project_solution = project_solution_builder
        //     .with_project_days(project_days)
        //     .build();

        // let shared_solution = SharedSolution {
        //     project: project_solution,
        //     ..SharedSolution::default()
        // };

        // // This is all a complete mess. I think that we should really think about
        // completing all this code // and then proceed to the next step.
        // let arc_swap_shared_solution =
        //     ArcSwapSharedSolution(ArcSwap::from_pointee(shared_solution));

        // let mut weekly_solution = WeeklySolution::new(&weekly_parameters);

        // weekly_solution
        //     .weekly_scheduled_work_orders
        //     .insert(work_order_number, Some(periods[0].clone()));

        // let mut weekly_algorithm = Algorithm::new(
        //     &Id::default(),
        //     weekly_solution,
        //     weekly_parameters,
        //     arc_swap_shared_solution.into(),
        // );

        // let operational_resource_0 = OperationalResource::new("OP_TEST_0",
        // Work::from(30.0), vec![     Resources::MtnMech,
        //     Resources::MtnElec,
        //     Resources::Prodtech,
        // ]);

        // weekly_algorithm
        //     .solution
        //     .weekly_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);

        // weekly_algorithm
        //     .update_based_on_shared_solution()
        //     .unwrap();

        // weekly_algorithm
        //     .unschedule_specific_work_order(work_order_number)
        //     .unwrap();
        // assert_eq!(
        //     *weekly_algorithm
        //         .solution
        //         .weekly_scheduled_work_orders
        //         .get(&work_order_number)
        //         .unwrap(),
        //     None
        // );
        Ok(())
    }

    #[test]
    fn test_period_clone_equality()
    {
        let period_1 = Period::from_str("2023-W47-48").unwrap();
        let period_2 = Period::from_str("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }

    // #[test]
    // fn test_update_loadings()
    // {
    //     let
    // }
}
