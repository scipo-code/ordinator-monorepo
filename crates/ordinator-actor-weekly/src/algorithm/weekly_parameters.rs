use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::worker_environment::WeeklyOptions;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Serialize;

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
}

impl Parameters for WeeklyParameters
{
    type Key = WorkOrderNumber;
    type Options = WeeklyOptions;

    fn from_scheduling_hypergraph(
        _id: &ActorCompositeId,
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
        _options: &Self::Options,
    ) -> Result<Self>
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();

        // Build work order parameters from the hypergraph view
        let weekly_work_order_parameters: HashMap<WorkOrderNumber, WeeklyWorkOrderParameter> =
            weekly_view
                .work_orders
                .iter()
                .map(|(&won, wo_view)| {
                    let locked_in_period = match &wo_view.assigned_period {
                        Some(period) => WhereIsWorkOrder::Weekly(period.clone()),
                        None => WhereIsWorkOrder::NotScheduled,
                    };

                    let excluded_periods = wo_view.excluded_periods.clone();

                    // Latest period is derived from basic_start_date: find the last
                    // period whose start date is on or after the work order's basic start
                    let latest_period = weekly_view
                        .periods
                        .iter()
                        .rev()
                        .find(|p| p.contains_date(wo_view.basic_start_date))
                        .or(weekly_view.periods.last())
                        .cloned()
                        .expect("There should always be at least one period");

                    // Work load: aggregate work_remaining by skill from activities
                    let mut work_load: HashMap<Skill, Work> = HashMap::new();
                    for activity in &wo_view.activities {
                        *work_load.entry(activity.required_skill).or_default() +=
                            activity.work_remaining;
                    }

                    // Weight derived from total work remaining
                    let weight = work_load
                        .values()
                        .fold(Work::from(0.0), |acc, w| acc + *w)
                        .to_f64() as i64;

                    (
                        won,
                        WeeklyWorkOrderParameter {
                            locked_in_period,
                            excluded_periods,
                            latest_period,
                            weight: std::cmp::max(weight, 1),
                            work_load,
                        },
                    )
                })
                .collect();

        // Clustering: derive empty clustering since functional location data
        // is not available from the hypergraph
        let weekly_clustering = WeeklyClustering {
            inner: HashMap::new(),
        };

        // Build capacity from technician availability
        let mut weekly_capacity: HashMap<Period, HashMap<Skill, Work>> = HashMap::new();
        for period in &weekly_view.periods {
            let mut capacity_work: HashMap<Skill, Work> = HashMap::new();

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

                for &skill in &technician.skills {
                    *capacity_work.entry(skill).or_default() += work_contribution;
                }
            }

            weekly_capacity.insert(period.clone(), capacity_work);
        }

        Ok(Self {
            weekly_work_order_parameters,
            weekly_capacity: WeeklyResources(weekly_capacity),
            weekly_clustering,
            period_locks: HashSet::default(),
            weekly_periods: weekly_view.periods,
        })
    }

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
    pub fn new_empty() -> Self
    {
        Self {
            inner: HashMap::new(),
        }
    }
}
