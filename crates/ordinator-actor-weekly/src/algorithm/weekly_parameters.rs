use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::materials::MaterialToPeriod;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::ClusteringWeights;
use ordinator_scheduling_environment::work_order::ForcedWorkOrder;
use ordinator_scheduling_environment::work_order::ProjectForceType;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrderPolicies;
use ordinator_scheduling_environment::work_order::WorkOrders;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::WeeklyOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Serialize;
use tracing::info;

pub type LowerLimitWork = Work;
pub type UpperLimitWork = Work;

use super::WeeklyResources;

#[derive(Debug)]
pub struct WeeklyParameters
{
    pub weekly_work_order_parameters: HashMap<WorkOrderNumber, WeeklyWorkOrderParameter>,
    pub weekly_capacity: WeeklyResources,
    pub weekly_clustering: WeeklyClustering,
    pub period_locks: HashSet<Period>,

    // TODO #04 #00 #01: Create PeriodState enum that changes based on SystemClock
    pub weekly_periods: Vec<Period>,
    pub weekly_options: WeeklyOptions,
}

// TODO: Consider implementing a builder pattern for Parameters
impl Parameters for WeeklyParameters
{
    type Key = WorkOrderNumber;

    // The asset change introduced some tradeoffs that need consideration
    //
    // This function is correct now, the issue is getting the options
    // out of the Paramaters
    fn from_scheduling_hypergraph(
        id: &ActorCompositeId,
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
    ) -> Result<Self>
    {
        let asset = id.2.main_asset();

        // TODO [ ] - extract the work orders
        let weekly_view = &scheduling_hypergraph.extract_weekly_view();

        // let work_orders = &scheduling_hypergraph.work_orders;
        // let weekly_periods = &scheduling_hypergraph.time_environment.periods;
        // let days = &scheduling_hypergraph.time_environment.days;

        // TODO: Move actor specifications retrieval to a separate module
        // let actor_specifications = scheduling_hypergraph
        //     .worker_environment
        //     .actor_specification
        //     .get(id.asset())
        //     .unwrap();

        let weekly_options = actor_specifications.weekly_options();

        // TODO [ ] these are still required
        let work_order_configurations = &scheduling_hypergraph.work_order_policies;
        let material_to_period = &scheduling_hypergraph.material_repo.material_to_period;

        // Filter work orders for this asset that are released for scheduling
        // FIX this is no longer needed - each asset should have its own hypergraph
        //
        // // This is of no concern to the algorithm
        // let filter = work_orders
        //     .inner
        //     .iter()
        //     .filter(|(_, wo)| wo.functional_location().asset == *asset)
        //     .filter(|(_, wo)| wo.released_for_scheduling());

        // ISSUE #000: Critical to fix correctly
        let weekly_work_order_parameters = filter
            .map(|(won, wo)| {
                Ok((
                    *won,
                    WeeklyWorkOrderParameter::builder()
                        // TODO: Accept list of work order numbers instead of current implementation
                        .with_scheduling_environment(
                            wo,
                            weekly_periods,
                            days,
                            work_order_configurations,
                            material_to_period,
                        )?
                        .build(),
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, WeeklyWorkOrderParameter>>>()?;

        let weekly_clustering = WeeklyClustering::calculate_clustering_values(
            asset,
            work_orders,
            &scheduling_hypergraph.work_order_policies.clustering_weights,
        )?;

        // multiskill_group        = [[1, 6], [2, 8], [4, 20]]
        // multiskill_group_total  = repeat([6 ; 4 ; 7], 1, length(data.days))
        //
        // @constraint(model, multi_skill_sum[ms in 1:3, day in data.days],
        //                     sum(c[wc, day] for wc in multiskill_group[ms]) ==
        // multiskill_group_total[ms, day])
        //
        //  @constraint(model, [k in operations, day in data.days],
        //                      data.people_for_operation[k] * p[k, day] <=
        // c[data.operation_to_workcenter[k], day])

        let mut weekly_capacity: HashMap<
            Period,
            HashMap<Skill, (LowerLimitWork, Work, UpperLimitWork)>,
        > = HashMap::new();
        for period in &weekly_view.periods {
            let mut capacity_work_lower_limit: HashMap<Skill, Work> = HashMap::new();
            let mut capacity_work: HashMap<Skill, Work> = HashMap::new();
            let mut capacity_work_upper_limit: HashMap<Skill, Work> = HashMap::new();

            for technician in weekly_view.technicians.values() {
                let days_in_period = technician
                    .available_dates
                    .iter()
                    .filter(|date| period.contains_date(**date))
                    .count();

                if days_in_period == 0 {
                    continue;
                }

                let work_contribution = Work::from(6.0 * days_in_period as f64);

                if technician.skills.len() == 1 {
                    let skill = *technician.skills.iter().next().unwrap();
                    *capacity_work_lower_limit.entry(skill).or_default() += work_contribution;
                    *capacity_work.entry(skill).or_default() += work_contribution;
                    *capacity_work_upper_limit.entry(skill).or_default() += work_contribution;
                } else {
                    let first_skill = *technician.skills.iter().next().unwrap();
                    *capacity_work.entry(first_skill).or_default() += work_contribution;
                    for &skill in &technician.skills {
                        *capacity_work_upper_limit.entry(skill).or_default() += work_contribution;
                    }
                }
            }

            let mut period_capacity = HashMap::new();
            let all_skills: HashSet<Skill> = capacity_work_lower_limit
                .keys()
                .chain(capacity_work.keys())
                .chain(capacity_work_upper_limit.keys())
                .copied()
                .collect();

            for skill in all_skills {
                let lower = capacity_work_lower_limit
                    .remove(&skill)
                    .unwrap_or(Work::from(0.0));
                let work = capacity_work.remove(&skill).unwrap_or(Work::from(0.0));
                let upper = capacity_work_upper_limit
                    .remove(&skill)
                    .unwrap_or(Work::from(0.0));
                period_capacity.insert(skill, (lower, work, upper));
            }

            weekly_capacity.insert(period.clone(), period_capacity);
        }

        Ok(Self {
            weekly_work_order_parameters,
            weekly_capacity: WeeklyResources(weekly_capacity),
            weekly_clustering,
            // This is not a feature. You need to design a common---
            period_locks: HashSet::default(),
            weekly_periods: weekly_view.periods,
            weekly_options: weekly_options.clone(),
        })
    }

    // TODO: Create as Builder using functional approach
    // ISSUE #000: create-individual-parameters-for-each-actor
    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

pub type ClusteringValue = i64;

#[derive(Debug, PartialEq, Clone)]
pub struct WeeklyClustering
{
    pub inner: HashMap<(WorkOrderNumber, WorkOrderNumber), ClusteringValue>,
}

/// WARNING: Consider adding a generic parameter to support multiple
/// WeeklyParameter handling approaches
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct WeeklyWorkOrderParameter
{
    pub locked_in_period: WhereIsWorkOrder<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Period,

    pub weight: i64,
    // Weight derived from WeeklyOptions
    pub work_load: HashMap<Skill, Work>,
}

// TODO: Reformulate using Type State pattern for complex business variants
// ISSUE #000: introduce-type-state-pattern-to-handle-complex-business-variants
// ISSUE #000: read-learning-domain-driven-design
#[derive(Debug)]
pub struct WorkOrderParameterBuilder
{
    pub locked_in_period: WhereIsWorkOrder<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Option<Period>,
    pub weight: Option<u64>,
    // Weight derived from WeeklyOptions
    pub work_load: HashMap<Skill, Work>,
}

// TODO: Use this for testing the scheduling program
// enum WeeklyParameterStates {
//     Scheduled,
//     BasicStart,
//     VendorWithUnloadingPoint,
//     FMCMainWorkCenter,
// }

impl WeeklyParameters
{
    pub fn get_locked_in_period<'a>(&'a self, work_order_number: &'a WorkOrderNumber)
    -> &'a Period
    {
        let option_period = match self.weekly_work_order_parameters.get(work_order_number) {
            Some(weekly_parameter) => &weekly_parameter.locked_in_period,
            None => {
                panic!("Work order number {work_order_number:?} not found in WeeklyParameters")
            }
        };
        match option_period {
            WhereIsWorkOrder::Weekly(period) => period,
            WhereIsWorkOrder::Project(_) => panic!("This should not happen"),
            WhereIsWorkOrder::NotScheduled => panic!(
                "Work order number {work_order_number:?} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order",
            ),
        }
    }

    pub fn set_locked_in_period(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: Period,
    ) -> Result<()>
    {
        let optimized_work_order = match self
            .weekly_work_order_parameters
            .get_mut(&work_order_number)
        {
            Some(optimized_work_order) => optimized_work_order,
            None => bail!(
                "Work order number {:?} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.locked_in_period = WhereIsWorkOrder::Weekly(period);
        Ok(())
    }
}

impl WorkOrderParameterBuilder
{
    pub fn build(self) -> WeeklyWorkOrderParameter
    {
        if let WhereIsWorkOrder::Weekly(ref locked_in_period) = self.locked_in_period {
            assert!(!self.excluded_periods.contains(locked_in_period));
        }

        WeeklyWorkOrderParameter {
            locked_in_period: self.locked_in_period,
            excluded_periods: self.excluded_periods,
            latest_period: self
                .latest_period
                .expect("There should always be a latest period on a WeeklyWorkOrder"),
            weight: self
                .weight
                .expect("There should always a weight on a WeeklyWorkOrder")
                as i64,
            work_load: self.work_load,
        }
    }
}

impl WeeklyWorkOrderParameter
{
    pub fn builder() -> WorkOrderParameterBuilder
    {
        WorkOrderParameterBuilder {
            locked_in_period: WhereIsWorkOrder::NotScheduled,
            excluded_periods: HashSet::default(),
            latest_period: None,
            weight: None,
            work_load: HashMap::default(),
        }
    }
}

impl WeeklyClustering
{
    pub fn calculate_clustering_values(
        asset: &Asset,
        work_orders: &WorkOrders,
        clustering_weights: &ClusteringWeights,
    ) -> Result<Self>
    {
        let mut clustering_similarity = HashMap::new();
        let work_orders_data: Vec<_> = work_orders
            .inner
            .iter()
            .filter(|(_, wo)| &wo.functional_location().asset == asset)
            .map(|(number, work_order)| {
                let fl = &work_order.functional_location();
                (
                    number,
                    fl.asset.clone(),
                    fl.sector(),
                    fl.system(),
                    fl.subsystem(),
                    fl.equipment_tag(),
                )
            })
            .collect();

        // Calculate similarity for each pair of work orders
        for i in 0..work_orders_data.len() {
            for j in i..work_orders_data.len() {
                let (wo_num1, asset1, sector1, system1, subsystem1, tag1) = &work_orders_data[i];
                let (wo_num2, asset2, sector2, system2, subsystem2, tag2) = &work_orders_data[j];

                let similarity = {
                    let mut score = 0;
                    if asset1 == asset2 {
                        score += clustering_weights.asset;
                    }
                    if sector1 == sector2 && sector2.is_some() {
                        score += clustering_weights.sector;
                    }
                    if system1 == system2 && system2.is_some() {
                        score += clustering_weights.system;
                    }
                    if subsystem1 == subsystem2 && subsystem2.is_some() {
                        score += clustering_weights.subsystem;
                    }
                    if tag1 == tag2 && tag2.is_some() {
                        score += clustering_weights.equipment_tag;
                    }
                    score
                };

                clustering_similarity.insert((**wo_num1, **wo_num2), similarity as i64);
            }
        }
        Ok(WeeklyClustering {
            inner: clustering_similarity,
        })
    }
}

pub fn create_weekly_parameters(
    _work_orders: &WorkOrders,
    _periods: &[Period],
    _asset: &Asset,
) -> Result<HashMap<WorkOrderNumber, WeeklyWorkOrderParameter>>
{
    todo!()
}
