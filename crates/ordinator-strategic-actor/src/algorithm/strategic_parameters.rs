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
use serde::Serialize;
use tracing::info;

use super::WeeklyResources;

#[derive(Debug)]
pub struct WeeklyParameters
{
    pub strategic_work_order_parameters: HashMap<WorkOrderNumber, WorkOrderParameter>,
    pub strategic_capacity: WeeklyResources,
    pub strategic_clustering: WeeklyClustering,
    pub period_locks: HashSet<Period>,

    // TODO #04 #00 #01: Create PeriodState enum that changes based on SystemClock
    pub strategic_periods: Vec<Period>,
    pub strategic_options: WeeklyOptions,
}

// TODO: Consider implementing a builder pattern for Parameters
impl Parameters for WeeklyParameters
{
    type Key = WorkOrderNumber;

    // The asset change introduced some tradeoffs that need consideration
    fn from_source(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let asset = id.2.main_asset();

        let work_orders = &scheduling_environment.work_orders;

        let strategic_periods = &scheduling_environment.time_environment.periods;
        let days = &scheduling_environment.time_environment.days;

        // TODO: Move actor specifications retrieval to a separate module
        let actor_specifications = scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .unwrap();

        let strategic_options = actor_specifications.strategic_options();
        let work_order_configurations = &scheduling_environment.work_order_policies;
        let material_to_period = &scheduling_environment.material_repo.material_to_period;

        // Filter work orders for this asset that are released for scheduling
        let filter = work_orders
            .inner
            .iter()
            .filter(|(_, wo)| wo.functional_location().asset == *asset)
            .filter(|(_, wo)| wo.released_for_scheduling());

        // ISSUE #000: Critical to fix correctly
        let strategic_work_order_parameters = filter
            .map(|(won, wo)| {
                Ok((
                    *won,
                    // TODO #000001: Move time environment configuration into SchedulingEnvironment
                    // TODO #000002: Move work order parameters to temp_scheduling_environment_database
                    WorkOrderParameter::builder()
                        // TODO: Accept list of work order numbers instead of current implementation
                        .with_scheduling_environment(
                            wo,
                            strategic_periods,
                            days,
                            work_order_configurations,
                            material_to_period,
                        )?
                        .build(),
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, WorkOrderParameter>>>()?;

        let strategic_clustering = WeeklyClustering::calculate_clustering_values(
            asset,
            work_orders,
            &scheduling_environment
                .work_order_policies
                .clustering_weights,
        )?;

        // TODO: Decouple SchedulingEnvironment from WeeklyResources
        let strategic_capacity = WeeklyResources::from((scheduling_environment, id));

        Ok(Self {
            strategic_work_order_parameters,
            strategic_capacity,
            strategic_clustering,
            period_locks: HashSet::default(),
            strategic_periods: strategic_periods.clone(),
            strategic_options: strategic_options.clone(),
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

/// WARNING: Consider adding a generic parameter to support multiple WeeklyParameter handling approaches
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct WorkOrderParameter
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
        let option_period = match self.strategic_work_order_parameters.get(work_order_number) {
            Some(strategic_parameter) => &strategic_parameter.locked_in_period,
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
            .strategic_work_order_parameters
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
    // This builder is crucial for business logic. Add functions for each field and clarify their meanings.
    // Configs come from the higher-level Parameters implementation via SchedulingEnvironment.
    pub fn with_scheduling_environment(
        mut self,
        work_order: &WorkOrder,
        periods: &[Period],
        days: &[Day],
        work_order_configurations: &WorkOrderPolicies,
        // TODO: Move material_to_period out of the system
        material_to_period: &MaterialToPeriod,
    ) -> Result<Self>
    {
        // TODO: Use TypeState pattern to improve error handling
        let forced_work_order = work_order.forced_work_order(periods, days, material_to_period)?;

        info!(target: "developer", forced_work_order = ?forced_work_order);
        match forced_work_order {
            ForcedWorkOrder::Period(period) => {
                self.locked_in_period = WhereIsWorkOrder::Weekly(period.0);
                self.excluded_periods = period.1;
            }
            // Give project control via WhereIsWorkOrder::Project when forced to days
            ForcedWorkOrder::Days(days) => {
                match &days {
                    ProjectForceType::OnlyStartDay(day) => {
                        let period = periods
                            .iter()
                            .find(|per| per.contains_date(day.date))
                            .context("day should always be contained in the period")?
                            .clone();
                        self.locked_in_period = WhereIsWorkOrder::Project(period);
                    }
                    ProjectForceType::IndividualActivities(_items, _hash_sets) => todo!(),
                }
                // Collect excluded periods where all days are excluded
                let excluded_periods = periods
                    .iter()
                    .filter(|per| days.excluded_days(per))
                    .cloned()
                    .collect::<HashSet<Period>>();
                // TODO: Evaluate if this data structure is optimal
                self.excluded_periods = excluded_periods;
            }
            ForcedWorkOrder::Technician(_technician_include, _technician_exclude) => todo!(),
            ForcedWorkOrder::FreeWorkOrder => {
                self.locked_in_period = WhereIsWorkOrder::NotScheduled;
                self.excluded_periods =
                    work_order.find_excluded_periods(periods, material_to_period)
            }
        }

        self.weight = Some(
            work_order
                .work_order_value(work_order_configurations)
                .with_context(|| {
                    format!("Could not calculate the work_order_value for: {work_order}")
                })?,
        );

        self.work_load = work_order
            .work_order_load()
            .context("Could not determine the work order load")?;

        self.latest_period = Some(work_order.latest_allowed_finish_period(periods).clone());
        Ok(self)
    }

    pub fn build(self) -> WorkOrderParameter
    {
        if let WhereIsWorkOrder::Weekly(ref locked_in_period) = self.locked_in_period {
            assert!(!self.excluded_periods.contains(locked_in_period));
        }

        WorkOrderParameter {
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

impl WorkOrderParameter
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

pub fn create_strategic_parameters(
    _work_orders: &WorkOrders,
    _periods: &[Period],
    _asset: &Asset,
) -> Result<HashMap<WorkOrderNumber, WorkOrderParameter>>
{
    todo!()
}
