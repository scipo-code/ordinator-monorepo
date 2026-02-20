pub mod assert_functions;
pub mod strategic_parameters;
pub mod strategic_resources;
pub mod strategic_solution;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use itertools::Itertools;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::ProjectInterface;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_environment::worker_environment::WeeklyOptions;
use ordinator_scheduling_environment::Percent;
use priority_queue::PriorityQueue;
use rand::distr::weighted::Weight;
use rand::prelude::SliceRandom;
use rand::seq::IndexedRandom;
use strategic_parameters::WeeklyClustering;
use strategic_parameters::WeeklyParameters;
use strategic_resources::OperationalResource;
use strategic_resources::WeeklyResources;
use strategic_solution::WeeklyObjectiveValue;
use strategic_solution::WeeklySolution;
use strum::IntoEnumIterator;
use tracing::instrument;

use crate::messages::requests::WeeklyRequestResource;
use crate::messages::requests::WeeklyRequestScheduling;
use crate::messages::responses::WeeklyResponseResources;
use crate::messages::responses::WeeklyResponseScheduling;

// How would this look if made generic? impl Algorithm<WeeklySolution, WeeklyParameters, WeeklyAssertions> { }
// Note: Making behavior generic is important, as changes would be needed in 4 places with the current design.

#[derive(Debug)]
pub struct WeeklyAlgorithm<Ss>(
    pub Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>,
)
where
    WeeklySolution: Solution,
    WeeklyParameters: Parameters,
    Ss: SystemSolutions,
    Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>:
        AbLNSUtils;

impl<Ss> Deref for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target =
        Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>;

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
    Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>:
        AbLNSUtils<SolutionType = WeeklySolution>,
    WeeklySolution: Solution,
    WeeklyParameters: Parameters,
    Ss: SystemSolutions<Weekly = WeeklySolution>,
{
    type Algorithm =
        Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>;
    type Options = WeeklyOptions;

    /// Incorporates the system solution by updating internal state and force scheduling work orders
    fn incorporate_system_solution(&mut self) -> Result<bool>
    {

        let mut state_change = true;
        let periods = self.parameters.strategic_periods.clone();
        // Weekly loops over all parameters
        for (work_order_number, strategic_parameter) in self
            .parameters
            .strategic_work_order_parameters
            .clone()
            .iter()

        {
            // Project model takes precedence over the strategic
            let project_scheduled_period = self
                .loaded_system_solution
                .project_actor_solution()
                .ok()
                .and_then(|project_solution| {
                    project_solution.project_period(work_order_number, &periods)
                });

            let strategic_scheduled_period = self
                .solution
                .strategic_scheduled_work_orders
                .get_mut(work_order_number)
                .with_context(|| {
                    format!("{work_order_number:?}\nis not found in the WeeklyAlgorithm")
                })?;

            // If the [`ProjectAlgorithm`] have the [`WorkOrder`] the [`Weekly`]
            // should respect this, but not schedule it and use resources.
            if let Some(project_period) = project_scheduled_period {
                if *strategic_scheduled_period != WhereIsWorkOrder::Project(project_period.clone()) {
                    state_change = true
                };
                *strategic_scheduled_period = WhereIsWorkOrder::Project(project_period.clone())
            }

            if strategic_parameter.locked_in_period == strategic_scheduled_period.clone() {
                continue;
            } 
            state_change = true;
        }

        // CRUCIAL: Forced work orders are always part of the shared state in incorporate()
        // This function should determine what is forced and only update WeeklyParameters 

        self.force_schedule()?;
        Ok(state_change)
    }

    fn make_atomic_pointer_swap(&mut self)
    {
        // TODO: Optimize by using Cow or selectively cloning only needed fields
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            // Swap the strategic solution in shared state
            shared_solution.strategic_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::Objective,
        >,
    >
    {
        let mut new_objective_value =
            WeeklyObjectiveValue::new(&self.parameters.strategic_options);

        self.determine_urgency(&mut new_objective_value)
            .context("could not determine strategic urgency")?;

        self.determine_resource_penalty(&mut new_objective_value);

        self.determine_clustering(&mut new_objective_value)
            .context("Could not determine WeeklyObjective value")?;

        self.determine_percent_scheduled(&mut new_objective_value)?;

        new_objective_value.aggregate_objectives();

        // We always work on self and substitute remaining parts
        if new_objective_value.objective_value
            < self.solution.objective_value().objective_value
        {
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            Ok(ObjectiveValueType::Worse(new_objective_value))
        }
    }

    fn schedule(&mut self) -> Result<()>
    {

        // TODO: Refactor to separate schedule and shared_state_update concerns
        while !self.solution_intermediate.is_empty() {
            for period in self.parameters.strategic_periods.clone() {
                let (work_order_number, weight) = match self.solution_intermediate.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),

                    None => {
                        break;
                    }
                };

                // TODO: Review overloaded logic here
                let inf_work_order_number = self
                    .schedule_strategic_work_order(work_order_number, &period)
                    .with_context(|| {
                        format!("{work_order_number:?} could not be scheduled normally")
                    })?;

                if let Some(work_order_number) = inf_work_order_number {
                    if &period != self.parameters.strategic_periods.last().unwrap() {
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
        let strategic_work_orders = self.solution.every_work_order();

        let strategic_parameters = &self.parameters.strategic_work_order_parameters;

        let mut filtered_keys: Vec<_> = strategic_work_orders
            .iter()
            .filter(|(won, _)| {
                strategic_parameters
                    .get(won)
                    .unwrap()
                    .locked_in_period
                    .not_scheduled()
            })
            .map(|(&won, _)| won)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(
                &mut rng,
                self.parameters
                    .strategic_options
                    .number_of_removed_work_orders,
            )
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
                .strategic_work_order_parameters
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

    fn force_schedule(&mut self) -> Result<()> {


        // ISSUE #999 - Disabled temporarily
        return Ok(());

        // TODO: Derive WeeklyParameter::locked_in_period from SchedulingEnvironment
        let forced_work_orders: Vec<_> = self
            .parameters
            .strategic_work_order_parameters
            .iter()
            .filter(|e|e.1.locked_in_period.strategic_forced())
            .map(|e|{
                // TODO: Remove ForcedWorkOrder if unnecessary
                ForcedWorkOrder::Locked(*e.0)
            }).collect();

        for forced_work_order_numbers in &forced_work_orders {
            self.schedule_forced_strategic_work_order(forced_work_order_numbers)
                .with_context(|| {
                    format!("{forced_work_order_numbers:#?} could not be force scheduled")
                })?;
        }

        Ok(())
    }

    fn throttling(&self, throttling: &ordinator_configuration::throttling::Throttling) -> u64 {
        throttling.strategic_throttling
    }
}

impl<Ss> WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // pub fn swap_scheduled_work_orders(&mut self, rng: &mut impl rand::Rng) {
    //         let scheduled_work_orders: Vec<_> = self
    //             .strategic_solution
    //             .strategic_periods
    //             .keys()
    //             .cloned()
    //             .collect();
    //         let randomly_chosen = scheduled_work_orders.choose_multiple(rng,
    // 2).collect::<Vec<_>>();         unsafe {
    //             let scheduled_work_order_1 =
    // self.solution.strategic_periods.get_mut(randomly_chosen[0]).unwrap() as *mut
    // Option<Period>;             let scheduled_work_order_2 =
    // self.solution.strategic_periods.get_mut(randomly_chosen[1]).unwrap() as *mut
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

    fn strategic_capacity_by_resource(&self, resource: &Skill, period: &Period)
        -> Result<Work>
    {
        self.parameters
            .strategic_capacity
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    fn strategic_loading_by_resource(&self, resource: &Skill, period: &Period) -> Result<Work>
    {
        self.solution
            .strategic_loadings
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    #[allow(dead_code)]
    pub fn calculate_utilization(&self) -> Result<Vec<(i64, u64)>>
    {
        let mut utilization_by_period = Vec::new();

        for (index, period) in self.parameters.strategic_periods.iter().enumerate() {
            let mut intermediate_loading: f64 = 0.0;
            let mut intermediate_capacity: f64 = 0.0;
            for resource in Skill::iter() {
                let loading = self.strategic_loading_by_resource(&resource, period)?;
                let capacity = self.strategic_capacity_by_resource(&resource, period)?;

                intermediate_loading += loading.to_f64();
                intermediate_capacity += capacity.to_f64();
            }
            let percentage_loading =
                ((intermediate_loading / intermediate_capacity) * 100.0) as u64;
            utilization_by_period.push((index as i64, percentage_loading));
        }
        Ok(utilization_by_period)
    }

    fn determine_urgency(
        &mut self,
        strategic_objective_value: &mut WeeklyObjectiveValue,
    ) -> Result<()>
    {
        for (work_order_number, scheduled_period) in self.solution.every_work_order() {
            let optimized_period = match scheduled_period {
                WhereIsWorkOrder::Weekly(optimized_period) => optimized_period,
                // CRUCIAL: Objective should be based on Project solution
                WhereIsWorkOrder::Project(period) => period,

                WhereIsWorkOrder::NotScheduled => self
                    .parameters
                    .strategic_periods
                    .last()
                    .context("There should always be a last .parameters.eriod")?,
            };

            let work_order_latest_allowed_finish_period = &self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .expect("WeeklyParameter should always be available for the WeeklySolution")
                .latest_period;

            let non_zero_period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );

            let work_order_value = self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .unwrap()
                .weight;

            let period_penalty = non_zero_period_difference * work_order_value;

            strategic_objective_value.urgency.1.checked_add_assign(&period_penalty)
                .ok()
                .with_context(|| format!("Overflow on the strategic urgency.\nperiod penalty: {period_penalty}\nperiod difference: {non_zero_period_difference}\nwork_order_value: {work_order_value}"))?;
        }
        Ok(())
    }

    fn determine_clustering(
        &mut self,
        strategic_objective_value: &mut WeeklyObjectiveValue,
    ) -> anyhow::Result<()>
    {
        for period in &self.parameters.strategic_periods {
            // Precompute scheduled work orders for the current period
            let scheduled_work_orders_by_period: Vec<_> = self
                .solution
                .every_work_order()
                .iter()
                .filter_map(|(won, where_is_period)| {
                    match where_is_period {
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
                    }
                })
                .collect();

            // Cache references to clustering inner map
            let clustering_inner = &self.parameters.strategic_clustering.inner;

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
                    strategic_objective_value.clustering_value.1 +=
                        *clustering_value_for_work_order_pair;
                }
            }
        }
        Ok(())
    }

    // Calculate resource penalty based on total exceeded hours
    fn determine_resource_penalty(
        &mut self,
        strategic_objective_value: &mut WeeklyObjectiveValue,
    )
    {
        for (resource, periods) in &self.parameters.strategic_capacity.0 {
            let capacity: f64 = periods.iter().map(|ele| ele.1.total_hours.to_f64()).sum();
            let loading: f64 = self
                .solution
                .strategic_loadings
                .0
                .get(resource)
                .unwrap()
                .iter()
                .map(|ele| ele.1.total_hours.to_f64())
                .sum();

            if loading - capacity > 0.0 {
                strategic_objective_value.resource_penalty.1 += (loading - capacity) as i64
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
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap();
        let work_load = &work_order_parameter.work_load;
        let locked_in_period = &work_order_parameter.locked_in_period;
        let strategic_loadings = self
            .solution
            .strategic_loadings
            .clone()
            .0
            .get(period)
            .unwrap()
            .clone();
        let strategic_capacity = self
            .parameters
            .strategic_capacity
            .0
            .get(period)
            .unwrap()
            .clone();
        ensure!(combined_loadings(work_load, &strategic_loadings).iter().all(|(res, work)| work >= work_load.get(res).unwrap()), "The amount of work loaded into the schedule and the work_load of the work order does not match.\n\
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
            combined_loadings(work_load, &strategic_loadings),
            work_load.clone().into_values().sum::<Work>(),
            combined_loadings(work_load, &strategic_capacity),
            work_load,
            locked_in_period,
            period,
            self.solution_intermediate.len(),
            Location::caller(),
        );
        Ok(())
    }

    fn determine_percent_scheduled(&self, new_objective_value: &mut WeeklyObjectiveValue) -> Result<()>{

            let total_work_orders = self.parameters.strategic_work_order_parameters.len() as u64;
            let scheduled_work_orders  = self.solution.strategic_scheduled_work_orders.iter().filter(|(_, v)| {
                match v {
                    WhereIsWorkOrder::Weekly(_period) => true,
                    WhereIsWorkOrder::Project(_) => false,
                    WhereIsWorkOrder::NotScheduled => false,
                }
            }).count() as u64;

            new_objective_value.percent_scheduled.1 = Percent::new(scheduled_work_orders, total_work_orders).context("percent scheduled could not be calculated")?;
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

// TODO: Move this trait to ProjectSolution interface (defines "Metavariables" from the paper)
pub trait WeeklyUtils
{
    fn schedule_strategic_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>>;

    fn schedule_forced_strategic_work_order(
        &mut self,
        force_schedule_work_order: &ForcedWorkOrder,
    ) -> Result<()>;

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool;

    /// This function updates the WeeklyResources based on the a provided
    /// loading.
    fn update_loadings(
        &mut self,
        strategic_resources: WeeklyResources,
        load_operation: LoadOperation,
    );

    fn determine_best_permutation(
        &self,
        work_load: HashMap<Skill, Work>,
        period: &Period,
        schedule: ScheduleWorkOrder,
    ) -> Result<Option<WeeklyResources>>;
}
// TODO: Use binary heap instead
impl<Ss> WeeklyUtils for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // TODO: Rely on interface instead. This function should determine period for project work order's first day

    fn schedule_strategic_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>>
    {
        let strategic_parameter = self
            .parameters
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .clone();

        let work_load: &HashMap<_, _> = &strategic_parameter
            .work_load
            .iter()
            .map(|e| (*e.0, e.1.round()))
            .collect();

        if strategic_parameter.excluded_periods.contains(period) {
            return Ok(Some(work_order_number));
        }

        if self.parameters.period_locks.contains(period) {
            return Ok(Some(work_order_number));
        }

        let resource_use_option = self
            .determine_best_permutation(work_load.clone(), period, ScheduleWorkOrder::Normal)
            .with_context(|| {
                format!(
                    "{:?}\nfor period\n{:#?}\ncould not be {:?}",
                    work_order_number,
                    period,
                    ScheduleWorkOrder::Normal
                )
            })?;

        let resource_use = match resource_use_option {
            Some(resource_use_inner) => resource_use_inner,
            None => return Ok(Some(work_order_number)),
        };

        ensure!(
            resource_use
                .0
                .get(period)
                .unwrap_or(&HashMap::from([(
                    "Dummy".to_string(),
                    OperationalResource::default()
                )]))
                .values()
                .fold(Work::from(0.0), |acc, e| acc + e.total_hours)
                == work_load.clone().into_values().sum(),
            "Calculated resources: {:#?}\nWork load: {:#?}",
            resource_use
                .0
                .get(period)
                .unwrap()
                .values()
                .fold(Work::from(0.0), |acc, e| acc + e.total_hours),
            work_load.clone().into_values().sum::<Work>()
        );

        let previous_period = self
            .solution
            .set_work_order_to_strategic(work_order_number, period.clone());

        ensure!(
            previous_period.as_ref().unwrap().not_scheduled(),
            "Previous period: {:#?}\nNew period: {:#?}\nWeeklyParameter: {:#?}\nfile: {}\nline: {}",
            &previous_period,
            period,
            strategic_parameter,
            file!(),
            line!()
        );

        // ensure!(resource_use.sum() == work_load.iter().sum())
        resource_use.assert_well_shaped_resources()?;
        self.update_loadings(resource_use.clone(), LoadOperation::Add);
        self.assert_work_load_to_loading(work_order_number, period)
            .with_context(|| {
                format!(
                    "Calculated resource use: {:#?}\nLocation: {}",
                    resource_use,
                    Location::caller()
                )
            })?;

        Ok(None)
    }

    // Ensures forced work orders are scheduled in correct order using template trait pattern
    fn schedule_forced_strategic_work_order(
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
            .set_work_order_to_strategic(*work_order_number, locked_in_period.clone())
            .with_context(|| {
                format!(
                    "Could not fully update {:#?} in {}",
                    force_schedule_work_order, &locked_in_period
                )
            })?;

        let work_load = self
            .parameters
            .strategic_work_order_parameters
            .get(force_schedule_work_order.work_order_number())
            .unwrap()
            .work_load
            .clone();

        let strategic_resources = self
            .determine_best_permutation(work_load, &locked_in_period, ScheduleWorkOrder::Forced)
            .with_context(|| format!("{:?}\ncould not be\n{:#?}", force_schedule_work_order, ScheduleWorkOrder::Forced))?
            .expect("It should always be possible to determine a resource permutation for a forced work order");

        strategic_resources.assert_well_shaped_resources()?;

        self.update_loadings(strategic_resources, LoadOperation::Add);
        Ok(())
    }

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool
    {
        self.solution
            .every_work_order()
            .get(work_order_number)
            .expect("This should always be initialized")
            .strategic_forced()
    }

    /// Updates WeeklyResources based on the provided loading
    fn update_loadings(
        &mut self,
        strategic_resources: WeeklyResources,
        load_operation: LoadOperation,
    )
    {
        // TODO: Refactor to handle changes correctly without permutation loop 
        for (period, operational_resources) in strategic_resources.0 {
            for (operational_id, operational_resource_loading) in operational_resources {
                assert!(operational_resource_loading.skill_hours.iter().all(|e| e.1 == &operational_resource_loading.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource_loading, std::panic::Location::caller());
                match load_operation {
                    LoadOperation::Add => {
                        self
                            .solution
                            .strategic_loadings
                            .0
                            .get_mut(&period)
                            .expect("All Periods should be initialized at this point")
                            // What happens if the value is not present? Should we simply insert it?
                            .entry(operational_id.clone())
                            .and_modify(|e| {
                                e.total_hours += operational_resource_loading.total_hours;
                                for (skill, hours) in &operational_resource_loading.skill_hours {
                                    e
                                        .skill_hours
                                        .entry(*skill) .and_modify(|wor| *wor += hours)
                                        .or_insert(*hours);
                                }

                            })
                            .or_insert(operational_resource_loading);

                    }
                    LoadOperation::Sub => {
                        let strategic_loading = self
                            .solution
                            .strategic_loadings
                            .0
                            .get_mut(&period)
                            .expect("All Periods should be initialized at this point")
                            .get_mut(&operational_id)
                            .unwrap();

                            strategic_loading.total_hours -= operational_resource_loading.total_hours;
                            for (skill, hours) in &operational_resource_loading.skill_hours {
                                strategic_loading
                                    .skill_hours
                                    .entry(*skill)
                                    .and_modify(|wor| *wor -= hours)
                                    .or_insert((*hours).negate());
                            }

                    }
                }
            }
        }
    }

    /// Determines the best permutation of work order assignments to technicians
    ///
    /// Returns whether a feasible permutation exists and the loading to apply
    fn determine_best_permutation(
        &self,
        work_load: HashMap<Skill, Work>,
        period: &Period,
        schedule: ScheduleWorkOrder,
    ) -> Result<Option<WeeklyResources>>
    {
        let mut rng = rand::rng();
        let mut best_total_excess = Work::from(-999999999.0);
        let mut best_work_order_resource_loadings = WeeklyResources::default();

        let strategic_capacity_resources = self
            .parameters
            .strategic_capacity
            .0
            .get(period)
            .context("There should always be a dummy resource that can soak excess")?;

        if matches!(schedule, ScheduleWorkOrder::Normal)
            && !work_load.keys().collect::<HashSet<&Skill>>().is_subset(
                &strategic_capacity_resources
                    .values()
                    .flat_map(|ele| ele.skill_hours.keys())
                    .collect::<HashSet<&Skill>>(),
            )
        {
            return Ok(None);
        }

        let strategic_loading_resources: HashMap<String, OperationalResource> = self
            .solution
            .strategic_loadings
            .0
            .get(period)
            .cloned()
            .unwrap_or_else(HashMap::new);

        // Difference between capacity and loading: positive means capacity available, negative means overloaded
        let difference_resources = determine_difference_resources(
            strategic_capacity_resources,
            &strategic_loading_resources,
        );

        for operational_resource in difference_resources.values() {
                ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());
        }
        for operational_resource in strategic_capacity_resources.values() {
                ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());
        }
        for operational_resource in strategic_loading_resources.values() {
                ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\nScheduleWorkOrder: {:?}\n{}", operational_resource, schedule, std::panic::Location::caller());
        }
        // Try 10 different technician permutations
        let mut error_for_unschedule = HashSet::new();
        let mut store_strategic_resources_options = vec![];
        for _ in 0..10 {
            let mut technician_permutation = difference_resources
                .clone()
                .into_values()
                .collect::<Vec<_>>();
            technician_permutation.shuffle(&mut rng);

            // Try 10 different work_load permutations
            for _ in 0..10 {
                let mut work_load_permutation = work_load.clone().into_iter().collect::<Vec<_>>();

                work_load_permutation.shuffle(&mut rng);

                error_for_unschedule.insert(work_load_permutation.clone());
                let strategic_resource_loadings_option = match schedule {
                    ScheduleWorkOrder::Normal => determine_normal_work_order_resource_loadings(
                        period,
                        &mut technician_permutation,
                        &mut work_load_permutation,
                    )
                    .with_context(|| {
                        "An error occured while calculating the normal scheduling loading"
                            .to_string()
                    })?,
                    ScheduleWorkOrder::Forced => {
                        let strategic_resource_loadings_option =
                            determine_forced_work_order_resource_loadings(
                                period,
                                &mut best_total_excess,
                                &mut best_work_order_resource_loadings,
                                &mut technician_permutation,
                                &mut work_load_permutation,
                            )
                            .context(
                                "incorrectly determine forced work order loadings\n{period}",
                            )?;

                        assert_work_load_equal_to_strategic_resource(
                            period,
                            &best_work_order_resource_loadings,
                            &work_load,
                            LoadOperation::Add,
                        )
                        .with_context(|| format!("file: {}\nline: {}", file!(), line!()))?;

                        if let Some(ref strategic_resource_loading) =
                            strategic_resource_loadings_option
                        {
                            assert_work_load_equal_to_strategic_resource(
                                period,
                                strategic_resource_loading,
                                &work_load,
                                LoadOperation::Add,
                            )
                            .with_context(|| format!("file: {}\nline: {}", file!(), line!()))?;
                        }

                        strategic_resource_loadings_option
                    }

                    // TODO: Handle rounding issues
                    ScheduleWorkOrder::Unschedule => {
                        // TODO: Remake combined_loadings to handle individual resources
                        // TODO: Fix the rounding issue
                        // Ensure strategic_loading is higher than work_load after each schedule operation


                        combined_loadings(&work_load, &strategic_loading_resources)
                            .iter()
                            .try_for_each(|(res, work)| {
                                let allowed = 
                                    work_load.get(res).cloned()
                                        .unwrap_or(Work::from(0.0));
                                        // .with_context(||format!("Resource: {res} is missing from the work_load: {work_load:#?}"))?;                                
                                ensure!( work >= &allowed, "The amount of work loaded into the schedule and the work_load of the work order does not match.\n\
                                    possible errors:\n\
                                    * Rounding error\n\
                                    * Calculation error\n\
                                    * Timing error in either pointer swaps or user-input message\n\
                                    combined_loadings: {:#?}\n\
                                    combined_work_load: {:#?}\n\
                                    work_load: {:#?}\n\
                                    Location: {}",
                                    combined_loadings(&work_load, &strategic_loading_resources),
                                    work_load.clone().into_values().sum::<Work>(),
                                    work_load,
                                    Location::caller(),
                                );
                                Ok(())
                            })?;

                        let order_map: HashMap<_, _> = technician_permutation
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (v.id, i))
                            .collect();

                        let strategic_resources_loading_vec: Vec<_> = strategic_loading_resources
                            .clone()
                            .into_iter()
                            .sorted_by_key(|ele| order_map.get(&ele.0))
                            .map(|ele| ele.1)
                            .collect();

                        // NOTE [ ]
                        // You can make the architecture even cleaner, if you remove the parameters,
                        // and simply define the parameters as a function
                        // directly on the `SchedulingEnvironment` you can really achieve a lot
                        // of state reduction.
                        //
                        // `Fn(SchedulingEnvironment) -> NumberOfPeople`
                        // `Fn(SchedulingEnvironment) -> work_order_load`
                        //
                        // The main issue with this is that the code will have to trade state for
                        // computation.
                        let strategic_resources_option =
                            determine_unschedule_work_resource_loadings(
                                period,
                                &strategic_resources_loading_vec,
                                &mut work_load_permutation,
                            ).with_context(||format!("Error in resource calculation\n{period:#?}\nWeekly loadings: {strategic_loading_resources:#?}\nWeekly capacities: {strategic_capacity_resources:#?}\nWork load: {work_load:?}"))?;

                        store_strategic_resources_options.push(strategic_resources_option.clone());

                        if let Some(ref strategic_resources) = strategic_resources_option {
                            assert_work_load_equal_to_strategic_resource(
                                period,
                                strategic_resources,
                                &work_load,
                                LoadOperation::Sub,
                            )
                            .with_context(|| {
                                format!(
                                    "strategic_resources_loading: {:#?}\nfile: {}\nline: {}",
                                    &strategic_loading_resources,
                                    file!(),
                                    line!()
                                )
                            })?;
                        }
                        strategic_resources_option
                    }
                };

                // If None, resource constraints not met; try next work_load_permutation
                let strategic_resource_loadings = match strategic_resource_loadings_option {
                    Some(strategic_resource_loadings) => strategic_resource_loadings,
                    None => continue,
                };

                match schedule {
                    ScheduleWorkOrder::Normal => {
                        if work_load_permutation
                            .iter()
                            .all(|(_, wor)| wor == &Work::from(0.0))
                        {
                            return Ok(Some(strategic_resource_loadings));
                        }
                    }
                    ScheduleWorkOrder::Forced => {
                        let equal_resources =
                            // Verify work_load skills are subset of available resources
                            work_load
                                .keys()
                                .collect::<HashSet<&Skill>>() 
                                .is_subset(
                                    &best_work_order_resource_loadings
                                        .0
                                        .get(period)
                                        .with_context(|| format!("{:#?}\nnot found in\n{}\n{}\n{}", period, std::any::type_name::<WeeklyResources>(), file!(), line!()))?
                                        .values()
                                        .flat_map(|ele| ele.skill_hours.keys())
                                        .collect::<HashSet<&Skill>>()
                                    );

                        ensure!(
                            equal_resources,
                            format!(
                                "{:#?}\n{:#?}\nfile: {}\nline: {}",
                                best_work_order_resource_loadings,
                                work_load,
                                file!(),
                                line!()
                            )
                        );
                        return Ok(Some(best_work_order_resource_loadings));
                    }
                    // Unscheduling is always possible
                    ScheduleWorkOrder::Unschedule => {
                        ensure!(combined_loadings(&work_load, &strategic_loading_resources).iter().all(|(res, work)| work >= work_load.get(res).unwrap()), "The amount of work loaded into the schedule and the work_load of the work order does not match.\n\
                            possible errors:\n\
                            * Rounding error\n\
                            * Calculation error\n\
                            * Timing error in either pointer swaps or user-input message\n\
                            combined_loadings: {:#?}\n\
                            combined_work_load: {:#?}\n\
                            work_load: {:#?}\n\
                            Location: {}",
                            combined_loadings(&work_load, &strategic_loading_resources),
                            work_load.clone().into_values().sum::<Work>(),
                            work_load,
                            Location::caller(),
                        );
                        assert_work_load_equal_to_strategic_resource(
                            period,
                            &strategic_resource_loadings,
                            &work_load,
                            LoadOperation::Sub,
                        )
                        .with_context(|| format!("Location: {}", Location::caller()))?;

                        if work_load_permutation
                            .iter()
                            .all(|res_wor| res_wor.1 == Work::from(0.0))
                        {
                            return Ok(Some(strategic_resource_loadings));
                        }
                    }
                }
            }
        }
        // TODO: Refactor return logic
        match schedule {
            ScheduleWorkOrder::Normal => Ok(None),
            ScheduleWorkOrder::Forced => Ok(Some(best_work_order_resource_loadings)),
            ScheduleWorkOrder::Unschedule => {
                // Verify resource constraints before returning

                ensure!(combined_loadings(&work_load, &strategic_loading_resources).iter().all(|(res, work)| work >= work_load.get(res).unwrap()), "The amount of work loaded into the schedule and the work_load of the work order does not match.\n\
                    possible errors:\n\
                    * Rounding error\n\
                    * Calculation error\n\
                    * Timing error in either pointer swaps or user-input message\n\
                    combined_loadings: {:#?}\n\
                    combined_work_load: {:#?}\n\
                    work_load: {:#?}\n\
                    Location: {}",
                    combined_loadings(&work_load, &strategic_loading_resources),
                    work_load.clone().into_values().sum::<Work>(),
                    work_load,
                    Location::caller(),
                );
                let filtered_strategic_loading_resources: Vec<_> = strategic_loading_resources
                    .iter()
                    .filter(|e| {
                        work_load
                            .keys()
                            .any(|res| e.1.skill_hours.contains_key(res))
                    })
                    .collect();

                let filtered_strategic_capacity_resources: Vec<_> = strategic_capacity_resources
                    .iter()
                    .filter(|e| {
                        work_load
                            .keys()
                            .any(|res| e.1.skill_hours.contains_key(res))
                    })
                    .collect();

                bail!(
                    "Unscheduling work order should always be possible\n\
                    {period:#?}\n\
                    Weekly loadings: {:#?}\n\
                    Weekly capacities: {:#?}\n\
                    Work load: {:#?}\n\
                    Stored strategic resources options: {:#?}\n\
                    Unsuccessful work_load_permutations: {:#?}",
                    filtered_strategic_loading_resources,
                    filtered_strategic_capacity_resources,
                    work_load,
                    store_strategic_resources_options,
                    error_for_unschedule
                )
            }
        }
    }
}

/// Combines skill-based loadings for a work order
fn combined_loadings(
    work_load: &HashMap<Skill, Work>,
    strategic_loading_resources: &HashMap<String, OperationalResource>,
) -> HashMap<Skill, Work>
{
    // HashMap of skill to total work for matching skills
    let mut strategic_loading: HashMap<Skill, Work, _> = HashMap::default();
    for resource in strategic_loading_resources.iter().filter(|(_, res)| {
        work_load
            .keys()
            .any(|res_work_load| res.skill_hours.contains_key(res_work_load))
    }) {
        for skill in resource.1.skill_hours.keys() {
            strategic_loading
                .entry(*skill)
                .and_modify(|e| *e += resource.1.total_hours)
                .or_insert(resource.1.total_hours);
        }
    }
    strategic_loading
}

fn assert_work_load_equal_to_strategic_resource(
    period: &Period,
    strategic_resource_loadings: &WeeklyResources,
    work_load: &HashMap<Skill, Work>,
    load_operation: LoadOperation,
) -> Result<()>
{
    let aggregate_strategic_resource = strategic_resource_loadings
            .0
            .get(period)
            .with_context(|| format!("{:#?}\nnot present. This probably means that nothing was {:#?}\nfile: {}\nline: {}", period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
            .iter()
            .fold(Work::from(0.0), |acc, or| acc + or.1.total_hours);

    let aggregate_work_load =
        work_load
            .values()
            .fold(Work::from(0.0), |acc, wor| match load_operation {
                LoadOperation::Add => acc + *wor,
                LoadOperation::Sub => acc - *wor,
            });

    let value = aggregate_work_load.equal(aggregate_strategic_resource);

    ensure!(
        value,
        format!(
            "Aggregate Work:\nWeeklyResources: {:#?}\nwork_load: {:#?}\n\n{:#?} {:#?}\nfile: {}\nline: {}",
            aggregate_strategic_resource,
            aggregate_work_load,
            work_load,
            strategic_resource_loadings,
            file!(),
            line!()
        )
    );
    Ok(())
}

fn determine_unschedule_work_resource_loadings(
    period: &Period,
    loading_resources: &[OperationalResource],
    work_load_permutation: &mut [(Skill, Work)],
) -> Result<Option<WeeklyResources>>
{
    let mut strategic_resources = WeeklyResources::default();
    let mut loading_resources_cloned = loading_resources.to_vec();
    for (resources, work) in work_load_permutation.iter_mut() {
        debug_assert!(loading_resources_cloned
            .iter()
            .flat_map(|or| or.skill_hours.keys())
            .collect::<HashSet<_>>()
            .contains(resources));

        // TODO: Determine optimal order to visit operational resources
        for operational_resource in loading_resources_cloned.iter_mut() {
            if !operational_resource.skill_hours.contains_key(resources) {
                continue;
            }
            ensure!(
                operational_resource
                    .skill_hours
                    .iter()
                    .all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours.\noperational_resource: {:#?}\n{}",
                operational_resource,
                std::panic::Location::caller()
            );

            if operational_resource.total_hours >= *work {
                operational_resource.total_hours -= *work;
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 -= *work);

                strategic_resources.update_load(
                    period,
                    *resources,
                    *work,
                    operational_resource,
                    LoadOperation::Sub,
                );

                *work = Work::from(0.0);
                ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());

                break;
            } else {
                *work -= operational_resource.total_hours;
                strategic_resources.update_load(
                    period,
                    *resources,
                    operational_resource.total_hours,
                    operational_resource,
                    LoadOperation::Sub,
                );

                operational_resource.total_hours = Work::from(0.0);
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 = Work::from(0.0));

                ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());
                continue;
            }
        }
    }

    // Return the strategic resources if all work_load is assigned
    if work_load_permutation
        .iter()
        .all(|ele| ele.1 == Work::from(0.0))
    {
        Ok(Some(strategic_resources))
    } else {
        Ok(None)
    }
}

/// This function determines the resource load for when a work order should be
/// forced into the schedule.
#[allow(unused_assignments)]
fn determine_forced_work_order_resource_loadings(
    period: &Period,
    best_total_excess: &mut Work,
    best_work_order_resource_loadings: &mut WeeklyResources,
    technician_permutation: &mut [OperationalResource],
    work_load_permutation: &mut [(Skill, Work)],
) -> Result<Option<WeeklyResources>>
{
    let mut work_order_resource_loadings = WeeklyResources::default();
    // TODO: Use Vec to handle multiple technicians
    for (resources, work) in work_load_permutation.iter_mut() {
        let mut qualified_technicians = technician_permutation
            .iter_mut()
            .filter(|or| or.skill_hours.keys().contains(resources))
            .collect::<Vec<_>>();

        for operational_resource in &qualified_technicians {
            ensure!(
                operational_resource
                    .skill_hours
                    .iter()
                    .all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours.\noperational_resource: {:#?}\n{}",
                operational_resource,
                std::panic::Location::caller()
            );
            
        }
        // If no qualified technicians, create dummy technician for skill
        let mut resources_store_dummy: Option<OperationalResource> = None;
        if qualified_technicians.is_empty() {
            let operational_resource = OperationalResource::new(
                &(resources.to_string() + "_dummy"),
                Work::from(0.0),
                HashSet::from([*resources]),
            );
            resources_store_dummy = Some(operational_resource);
            qualified_technicians.push(resources_store_dummy.as_mut().unwrap());
        };

        // TODO: Fill each technician individually; minimize degrees of freedom
        // TODO: Handle jobs larger than available resources
        let work_load_by_resources_by_technician = work
            .divide_work(
                qualified_technicians
                    .iter()
                    .map(|e| e.total_hours)
                    .collect(),
            )
            .context("Could not divide work among qualified technicians")?;

        for (work_load_by_technician, operational_resource) in work_load_by_resources_by_technician
            .iter()
            .zip(qualified_technicians)
        {
            ensure!(
                operational_resource
                    .skill_hours
                    .iter()
                    .all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\n{}",
                std::panic::Location::caller()
            );
            operational_resource.total_hours -= work_load_by_technician;
            operational_resource
                .skill_hours
                .iter_mut()
                .for_each(|(_, wor)| *wor -= work_load_by_technician);
            // TODO: Create dummy for skill if needed
            work_order_resource_loadings.update_load(
                period,
                *resources,
                *work_load_by_technician,
                operational_resource,
                LoadOperation::Add,
            );
        }
    }
    let total_excess = technician_permutation
        .iter()
        .map(|or| std::cmp::min(Work::from(0.0), or.total_hours))
        .reduce(|acc, wor| acc + wor)
        .expect("The Technician Permutation here should never be empty");

    if total_excess == Work::from(0.0) {
        *best_work_order_resource_loadings = work_order_resource_loadings.clone();
        return Ok(Some(work_order_resource_loadings));
    } else if total_excess > *best_total_excess {
        *best_work_order_resource_loadings = work_order_resource_loadings.clone();
        *best_total_excess = total_excess;
    }
    Ok(None)
}

fn determine_normal_work_order_resource_loadings(
    period: &Period,
    technician_permutation: &mut [OperationalResource],
    work_load_permutation: &mut Vec<(Skill, Work)>,
) -> Result<Option<WeeklyResources>>
{
    let mut work_order_resource_loadings = WeeklyResources::default();
    // Encode state in ensure!() macros to improve error handling and debugging
    let total_work_load = work_load_permutation
        .iter()
        .fold(Work::from(0.0), |acc, e| acc + e.1);

    // If there is no work in the operation simply return from it.
    if total_work_load == Work::from(0.0) {
        ensure!(
            work_load_permutation.iter().all(|e| e.1 == Work::from(0.0)),
            "Some of the individual work_loads were negative\nwork_load_permutation: {:#?}",
            work_load_permutation
        );
        return Ok(None);
    }
    if technician_permutation
        .iter()
        .fold(Work::from(0.0), |acc, e| acc + e.total_hours)
        == work_load_permutation
            .iter()
            .fold(Work::from(0.0), |acc, e| acc + e.1)
    {
        return Ok(None);
    }

    for operation_load in &mut *work_load_permutation {
        for operational_resource in technician_permutation.iter_mut() {
            ensure!(
                operational_resource
                    .skill_hours
                    .iter()
                    .all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours.\noperational_resource: {:#?}\n{}",
                operational_resource,
                std::panic::Location::caller()
            );
            if !operational_resource
                .skill_hours
                .keys()
                .contains(&operation_load.0)
            {
                continue;
            }

            if operational_resource.total_hours <= Work::from(0.0) {
                continue;
            }

            if operation_load.1 <= Work::from(0.0) {
                continue;
            }

            if operation_load.1 <= operational_resource.total_hours {
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|(_, wor)| *wor -= operation_load.1);
                operational_resource.total_hours -= operation_load.1;
                work_order_resource_loadings.update_load(
                    period,
                    operation_load.0,
                    operation_load.1.round(),
                    operational_resource,
                    LoadOperation::Add,
                );
                operation_load.1 = Work::from(0.0);

                ensure!(
                    operational_resource
                        .skill_hours
                        .iter()
                        .all(|e| e.1 == &operational_resource.total_hours),
                    "Each skill_hours should always be equal to the value of the total_hours\n{}",
                    std::panic::Location::caller()
                );

                break;
            } else {
                operation_load.1 -= operational_resource.total_hours;
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|(_res, wor)| *wor = Work::from(0.0));
                work_order_resource_loadings.update_load(
                    period,
                    operation_load.0,
                    operational_resource.total_hours.round(),
                    operational_resource,
                    LoadOperation::Add,
                );
                // PRINCIPLE: Failure is acceptable; not learning from it is not
                ensure!(
                    work_order_resource_loadings
                        .0
                        .get(period)
                        .unwrap()
                        .get(&operational_resource.id)
                        .unwrap()
                        .total_hours
                        == operational_resource.total_hours.round(),
                    "{:<30}: {:#?}\n\
                    {:<30}: {:#?}",
                    "operational_resource_total_hours",
                    operational_resource.total_hours,
                    "work_order_resource_loadings",
                    work_order_resource_loadings
                        .0
                        .get(period)
                        .unwrap()
                        .get(&operational_resource.id)
                        .unwrap()
                        .total_hours
                        .round(),
                );
                operational_resource.total_hours = Work::from(0.0);
                ensure!(
                    operational_resource
                        .skill_hours
                        .iter()
                        .all(|e| e.1 == &operational_resource.total_hours),
                    "Each skill_hours should always be equal to the value of the total_hours\n\
                    Total hours: {:?}\n\
                    Skill hours: {:#?}\n\
                    Location: {}",
                    &operational_resource.total_hours,
                    operational_resource.skill_hours,
                    std::panic::Location::caller()
                );
            }
        }

        if operation_load.1 != Work::from(0.0) {
            return Ok(None);
        }
    }
    ensure!(work_load_permutation.iter().all(|e| e.1.is_zero()));

    // TODO: Handle numerical issues centrally with custom error types
    ensure!(
        total_work_load
            == work_order_resource_loadings
                .0
                .get(period)
                .unwrap_or(&HashMap::from([(
                    "Dummy".to_string(),
                    OperationalResource::default()
                )]))
                .values()
                .fold(Work::from(0.0), |acc, e| acc + e.total_hours)
                .round(),
        "Total work_load: {:#?}\n\
        work_load_permutation_resources: {:#?}\n\
        work_order_resource_loadings: {:#?}\n\
        calculated_resources: {:#?}\n\
        Location: {}",
        total_work_load,
        work_load_permutation
            .iter()
            .map(|e| e.0)
            .collect::<Vec<_>>(),
        work_order_resource_loadings
            .0
            .get(period)
            .unwrap()
            .values()
            .fold(Work::from(0.0), |acc, e| acc + e.total_hours),
        work_order_resource_loadings.0.get(period).unwrap(),
        Location::caller(),
    );

    Ok(Some(work_order_resource_loadings))
}

fn determine_difference_resources(
    capacity_resources: &HashMap<String, OperationalResource>,
    loading_resources: &HashMap<String, OperationalResource>,
) -> HashMap<String, OperationalResource>
{
    let mut difference_resources = HashMap::new();
    for capacity in capacity_resources {
        let loading = loading_resources
            .get(capacity.0)
            .cloned()
            .unwrap_or_default();
        let total_hours = capacity.1.total_hours - loading.total_hours;
        let skills = capacity.1.skill_hours.clone().into_keys().collect();

        let operational_resource = OperationalResource::new(capacity.0, total_hours, skills);

        difference_resources.insert(capacity.0.clone(), operational_resource);
    }
    difference_resources
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
        _strategic_resources_request: WeeklyRequestResource,
    ) -> Result<WeeklyResponseResources>
    {
        // TODO: Implement resource state update logic
        // match strategic_resources_request {

        //     WeeklyRequestResource::GetLoadings {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let loading = &self.solution.strategic_loadings;

        //         let strategic_response_resources =
        //
        // WeeklyResponseResources::LoadingAndCapacities(loading.clone());
        //         Ok(strategic_response_resources)
        //     }
        //     WeeklyRequestResource::GetCapacities {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let capacities = &self.parameters.strategic_capacity;

        //         let strategic_response_resources =
        //
        // WeeklyResponseResources::LoadingAndCapacities(capacities.clone());
        //         Ok(strategic_response_resources)
        //     }
        //     WeeklyRequestResource::GetPercentageLoadings {
        //         periods_end: _,
        //         resources: _,
        //     } => {
        //         let capacities = &self.parameters.strategic_capacity;
        //         let loadings = &self.solution.strategic_loadings;

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
        strategic_scheduling_request: WeeklyRequestScheduling,
    ) -> Result<WeeklyResponseScheduling>
    {
        match strategic_scheduling_request {
            WeeklyRequestScheduling::Schedule(schedule_work_order) => {
                let period = self
                    .parameters
                    .strategic_periods
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
                    let strategic_parameter = self
                        .parameters
                        .strategic_work_order_parameters
                        .get_mut(&work_order_number)
                        .unwrap();
                    if strategic_parameter.excluded_periods.contains(&period) {
                        strategic_parameter.excluded_periods.remove(&period);
                    }
                    self.parameters
                        .set_locked_in_period(work_order_number, period.clone())
                        .context("could not set locked in period")?;
                    number_of_work_orders += 1;
                }

                Ok(WeeklyResponseScheduling::new(
                    number_of_work_orders,
                    period,
                ))
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
            let strategic_parameter = self
                .parameters
                .strategic_work_order_parameters
                .get(&work_order_number)
                .unwrap();

            let work_load = strategic_parameter.work_load.clone();

            let strategic_resources = self
                .determine_best_permutation(work_load, &unschedule_from_period, ScheduleWorkOrder::Unschedule)
                .with_context(|| format!("{:#?}\n{:#?}\nfor {:?}\nfile: {}\nline: {}", strategic_parameter, unschedule_from_period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
                .context("Determining the WeeklyResources associated with a unscheduling operation should always be possible")?;

            strategic_resources.assert_well_shaped_resources()?;
            self.update_loadings(strategic_resources, LoadOperation::Sub);
        }
        Ok(())
    }

    // TODO: Refactor populate_priority_queue
    pub fn populate_priority_queue(&mut self)
    {
        for work_order_number in self.solution.every_work_order().clone().keys() {
            let strategic_parameter = self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .expect(
                    "The WeeklyParameter should always be available for the WeeklySolution",
                );

            if strategic_parameter.locked_in_period.strategic_forced() {
                continue;
            }

            if self
                .solution
                .every_work_order()
                .get(work_order_number)
                .unwrap()
                .not_scheduled()
            {
                let strategic_work_order_weight = strategic_parameter.weight;
                self.solution_intermediate
                    .push(*work_order_number, strategic_work_order_weight);
            }
        }
    }
}

impl<Ss>
    From<Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, i64>, Ss>>
    for WeeklyAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(
        value: Algorithm<
            WeeklySolution,
            WeeklyParameters,
            PriorityQueue<WorkOrderNumber, i64>,
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
    use std::str::FromStr;

    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use strategic_parameters::WorkOrderParameter;

    use super::*;

    impl WorkOrderParameter
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
    fn test_determine_difference_resources()
    {
        let capacity_resource = HashMap::from([
            (
                "Test one".to_string(),
                OperationalResource::new("Test one", Work::from(5.0), HashSet::from([Skill::MtnMech])),
            ),
            (
                "Test two".to_string(),
                OperationalResource::new("Test two", Work::from(4.0), HashSet::from([Skill::MtnElec])),
            ),
            (
                "Test three".to_string(),
                OperationalResource::new("Test three", Work::from(3.0), HashSet::from([Skill::MtnScaf])),
            ),
        ]);
        let loading_resource = HashMap::from([
            (
                "Test one".to_string(),
                OperationalResource::new("Test one", Work::from(2.0), HashSet::from([Skill::MtnMech])),
            ),
            (
                "Test two".to_string(),
                OperationalResource::new("Test two", Work::from(2.0), HashSet::from([Skill::MtnElec])),
            ),
            (
                "Test three".to_string(),
                OperationalResource::new("Test three", Work::from(2.0), HashSet::from([Skill::MtnScaf])),
            ),
        ]);

        let difference = determine_difference_resources(&capacity_resource, &loading_resource);

        let difference_actual = HashMap::from([
            (
                "Test one".to_string(),
                OperationalResource::new("Test one", Work::from(3.0), HashSet::from([Skill::MtnMech])),
            ),
            (
                "Test two".to_string(),
                OperationalResource::new("Test two", Work::from(2.0), HashSet::from([Skill::MtnElec])),
            ),
            (
                "Test three".to_string(),
                OperationalResource::new("Test three", Work::from(1.0), HashSet::from([Skill::MtnScaf])),
            ),
        ]);

        assert_eq!(difference, difference_actual);
    }
    #[test]
    fn test_determine_best_permutation() {}

    #[test]
    fn test_update_load_1()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::MtnMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            HashSet::from([Skill::MtnMech, Skill::MtnElec, Skill::Prodtech]),
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = WeeklyResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Add,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(130.0)
        );
        assert!(strategic_resources
            .0
            .get(&period)
            .unwrap()
            .get(operational_id)
            .unwrap()
            .skill_hours
            .values()
            .all(|wor| *wor == Work::from(130.0)));
    }

    #[test]
    fn test_update_load_2()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::VenMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            HashSet::from([Skill::MtnMech, Skill::MtnElec, Skill::Prodtech]),
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = WeeklyResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Add,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(130.0)
        );
        assert!(strategic_resources
            .0
            .get(&period)
            .unwrap()
            .get(operational_id)
            .unwrap()
            .skill_hours
            .values()
            .all(|wor| *wor == Work::from(130.0)));
        assert!(strategic_resources
            .0
            .get(&period)
            .unwrap()
            .get(operational_id)
            .unwrap()
            .skill_hours
            .contains_key(&resource));
    }
    #[test]
    fn test_update_load_3()
    {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Skill::VenMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            HashSet::from([Skill::MtnMech, Skill::MtnElec, Skill::Prodtech]),
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = WeeklyResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Sub,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(70.0)
        );
        assert!(strategic_resources
            .0
            .get(&period)
            .unwrap()
            .get(operational_id)
            .unwrap()
            .skill_hours
            .values()
            .all(|wor| *wor == Work::from(70.0)));
        assert!(strategic_resources
            .0
            .get(&period)
            .unwrap()
            .get(operational_id)
            .unwrap()
            .skill_hours
            .contains_key(&resource));
    }

    #[test]
    #[should_panic]
    fn test_determine_normal_work_order_resource_loadings_0()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(30.0)),
            (Skill::MtnElec, Work::from(30.0)),
            (Skill::MtnScaf, Work::from(30.0)),
        ];

        // Ahh you should use your tests
        let strategic_resource_option = super::determine_normal_work_order_resource_loadings(
            &period,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .unwrap();

        assert!(strategic_resource_option.is_none());
    }
    #[test]
    fn test_determine_normal_work_order_resource_loadings_1() -> Result<()>
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(30.0)),
            (Skill::MtnElec, Work::from(30.0)),
            (Skill::MtnScaf, Work::from(20.0)),
        ];

        // Ahh you should use your tests
        let strategic_resource_option = super::determine_normal_work_order_resource_loadings(
            &period,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .context("Test failed")?;

        assert!(strategic_resource_option.is_none());
        Ok(())
    }

    #[test]
    fn test_determine_normal_work_order_resource_loadings_2()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(20.0)),
            (Skill::MtnElec, Work::from(20.0)),
            (Skill::MtnScaf, Work::from(20.0)),
        ];

        let strategic_resource_option = super::determine_normal_work_order_resource_loadings(
            &period,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .unwrap();

        assert!(strategic_resource_option.is_some());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(20.0),
            HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
        );

        let mut strategic_resource = WeeklyResources::default();
        strategic_resource.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resource.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resource, strategic_resource_option.unwrap());
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_1()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(30.0)),
            (Skill::MtnElec, Work::from(30.0)),
            (Skill::MtnScaf, Work::from(30.0)),
        ];

        let mut best_strategic_resource = WeeklyResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .with_context(|| {
            format!(
                "Incorrectly determined the forced work order loadings\n{}",
                std::panic::Location::caller()
            )
        })
        .unwrap();

        assert!(strategic_resources.is_none());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(50.0),
            HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
        );
        let mut strategic_resources = WeeklyResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resources, best_strategic_resource);
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_2()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(20.0)),
            (Skill::MtnElec, Work::from(20.0)),
            (Skill::MtnScaf, Work::from(20.0)),
        ];

        let mut best_strategic_resource = WeeklyResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources_option = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .unwrap();

        assert!(strategic_resources_option.is_some());
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(20.0),
            HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
        );
        let mut strategic_resources = WeeklyResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resources, strategic_resources_option.unwrap());
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_3()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(30.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(30.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(20.0)),
            (Skill::MtnElec, Work::from(20.0)),
            (Skill::MtnScaf, Work::from(20.0)),
            (Skill::VenMech, Work::from(20.0)),
        ];

        let mut best_strategic_resource = WeeklyResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources_option = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        )
        .unwrap();

        assert!(strategic_resources_option.is_some());
        // Is this the right way of doing things? I do not think that it is...
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(30.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(30.0),
            HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
        );
        let operational_resource_3 =
            OperationalResource::new("VEN-MECH_dummy", Work::from(20.0), HashSet::from([Skill::VenMech]));

        let mut strategic_resources = WeeklyResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_3);

        assert_eq!(strategic_resources, strategic_resources_option.unwrap());
    }

    #[test]
    fn test_determine_unschedule_work_order_loadings_1()
    {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut work_load_permutation = vec![
            (Skill::MtnMech, Work::from(20.0)),
            (Skill::MtnElec, Work::from(20.0)),
        ];

        let loading_resources = [
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(20.0),
                HashSet::from([Skill::MtnMech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(20.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
            ),
        ];

        let strategic_resources_option_calculated = determine_unschedule_work_resource_loadings(
            &period,
            &loading_resources,
            &mut work_load_permutation,
        )
        .unwrap();

        assert!(strategic_resources_option_calculated.is_some());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(-20.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(-20.0),
            HashSet::from([Skill::MtnScaf, Skill::MtnElec]),
        );
        let mut strategic_resources_manual = WeeklyResources::default();

        strategic_resources_manual
            .insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources_manual
            .insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(
            strategic_resources_manual,
            strategic_resources_option_calculated.unwrap()
        );
    }

    #[test]
    fn test_determine_unschedule_work_order_loadings_2() -> Result<()>
    {
        let period = Period::from_str("2026-W33-34").unwrap();

        let work_load_permutation = [
            (Skill::MtnMech, Work::from(2.0)),
            (Skill::Prodtech, Work::from(2.0)),
            (Skill::MtnInst, Work::from(2.0)),
            (Skill::MtnElec, Work::from(2.0)),
        ];

        let loading_resources = [
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(6.0),
                HashSet::from([Skill::MtnMech, Skill::Prodtech, Skill::MtnElec]),
            ),
            OperationalResource::new(
                "OP_TEST_2",
                Work::from(0.0),
                HashSet::from([Skill::MtnScaf, Skill::MtnRigg, Skill::MtnLagg]),
            ),
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(2.0),
                HashSet::from([Skill::MtnInst, Skill::MtnMech, Skill::MtnElec]),
            ),
        ];

        // We should readjust this to take in a Vec<T> instead of a HashMap<K, V>. That
        // is a much better approach for dealing with this. And then we reuse
        // the structure that we already have. QUESTION:
        // Can you combine the .permutation() with random access.
        let strategic_resources_option = determine_unschedule_work_resource_loadings(
            &period,
            &loading_resources,
            &mut work_load_permutation.clone(),
        )
        .unwrap();

        super::assert_work_load_equal_to_strategic_resource(
            &period,
            strategic_resources_option.as_ref().unwrap(),
            &HashMap::from(work_load_permutation),
            LoadOperation::Sub,
        )?;

        assert!(strategic_resources_option.is_some());

        // let operational_resource_1 = OperationalResource::new("OP_TEST_0",
        // Work::from(-20.0), vec![Resources::MtnMech, Resources::MtnElec]); let
        // operational_resource_2 = OperationalResource::new("OP_TEST_1",
        // Work::from(-20.0), vec![Resources::MtnScaf, Resources::MtnElec]); let
        // mut strategic_resources = WeeklyResources::default();

        // strategic_resources.insert_operatensureonal_resource(period.clone(),
        // operational_resource_1); strategic_resources.
        // insert_operational_resource(period.clone(), operational_resource_2);

        // assert_eq!(strategic_resources, strategic_resources_option.unwrap());

        //FIX MAKE A PROPTEST
        Ok(())
    }

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

        let mut strategic_resources = WeeklyResources::default();

        let operational_resource_0 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );

        strategic_resources.insert_operational_resource(periods[0].clone(), operational_resource_0);
        strategic_resources.insert_operational_resource(periods[1].clone(), operational_resource_1);

        // This way of making parameters needs to go away. Is the right call here to
        // simply delete the let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let mut strategic_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_parameter_1 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_1,
        // );

        // let strategic_parameter_2 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_2,
        // );

        // let strategic_parameter_3 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_3,
        // );

        // let work_order_number_1 = WorkOrderNumber(2200000001);
        // let work_order_number_2 = WorkOrderNumber(2200000002);
        // let work_order_number_3 = WorkOrderNumber(2200000003);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_1, strategic_parameter_1);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_2, strategic_parameter_2);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_3, strategic_parameter_3);

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let strategic_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_solution = WeeklySolution::new(&strategic_parameters);

        // Actor::builder().agent_id(&id).
        // scheduling_environment(scheduling_environment).algorithm(|con|con.id(&id).
        // parameters(options, scheduling_environment)).configurations();

        // let mut strategic_algorithm = Algorithm::new(
        //     &Id::default(),
        //     strategic_solution,
        //     strategic_parameters,
        //     ArcSwapSharedSolution::default().into(),
        // );

        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number_1, Some(periods[0].clone()));
        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number_2, Some(periods[1].clone()));
        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
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

        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);
        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[1].clone(), operational_resource_1);

        // let seed: [u8; 32] = [
        //     1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        // 21, 22, 23, 24,     25, 26, 27, 28, 29, 30, 31, 32,
        // ];

        // let rng = StdRng::from_seed(seed);

        // let strategic_options = WeeklyOptions {
        //     number_of_removed_work_order: 2,
        //     rng,
        //     urgency_weight: 1,
        //     resource_penalty_weight: 1,
        //     clustering_weight: 1,
        //     work_order_configurations: todo!(),
        //     material_to_period: todo!(),
        // };

        // strategic_algorithm.parameters.strategic_options = strategic_options;

        // strategic_algorithm.unschedule().expect(
        //     "It should always be possible to unschedule random work orders in the
        // strategic agent", );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000001))
        //         .unwrap(),
        //     Some(Period::from_str("2023-W47-48").unwrap())
        // );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000002))
        //         .unwrap(),
        //     None
        // );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
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
        let mut strategic_resources = WeeklyResources::default();

        let operational_resource_0 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            HashSet::from([Skill::MtnMech, Skill::MtnElec]),
        );

        strategic_resources.insert_operational_resource(periods[0].clone(), operational_resource_0);

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Weekly", vec![], vec![Asset::Unknown]);

        // let mut strategic_parameters = WeeklyParameters::new(
        //     &id,
        //     WeeklyOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_parameter = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     periods[0].clone(),
        //     1000,
        //     HashMap::from([(Resources::MtnMech, Work::from(5.0))]),
        // );

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number, strategic_parameter);

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

        // let mut strategic_solution = WeeklySolution::new(&strategic_parameters);

        // strategic_solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number, Some(periods[0].clone()));

        // let mut strategic_algorithm = Algorithm::new(
        //     &Id::default(),
        //     strategic_solution,
        //     strategic_parameters,
        //     arc_swap_shared_solution.into(),
        // );

        // let operational_resource_0 = OperationalResource::new("OP_TEST_0",
        // Work::from(30.0), vec![     Resources::MtnMech,
        //     Resources::MtnElec,
        //     Resources::Prodtech,
        // ]);

        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);

        // strategic_algorithm
        //     .update_based_on_shared_solution()
        //     .unwrap();

        // strategic_algorithm
        //     .unschedule_specific_work_order(work_order_number)
        //     .unwrap();
        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
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
