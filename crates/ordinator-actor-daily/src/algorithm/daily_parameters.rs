use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::Inspect;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::DailyOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;

pub struct DailyParameters
{
    pub daily_work_orders: HashMap<WorkOrderNumber, HashMap<ActivityNumber, DailyParameter>>,
    pub daily_periods: Vec<Period>,
}

// TODO: Add assertions on vector elements
impl std::fmt::Debug for DailyParameters
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Number of WorkOrderActivities: {}\n\
                Daily periods: {:#?}",
            self.daily_work_orders.len(),
            self.daily_periods,
        )
    }
}

impl Parameters for DailyParameters
{
    type Key = WorkOrderActivity;
    type Options = DailyOptions;

    fn from_scheduling_hypergraph(
        _id: &ActorCompositeId,
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
        _options: &Self::Options,
    ) -> Result<Self>
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();

        let mut daily_parameters = HashMap::new();

        for (&work_order_number, wo_view) in &weekly_view.work_orders {
            let mut inner_map = HashMap::new();
            for activity in &wo_view.activities {
                let daily_parameter = DailyParameter::new(
                    activity.required_skill,
                    activity.number_of_people,
                    activity.work_remaining,
                );
                inner_map.insert(activity.activity_number, daily_parameter);
            }

            let _assert_option = daily_parameters.insert(work_order_number, inner_map);
            assert!(_assert_option.is_none());
        }

        // Use all available periods from the hypergraph
        let daily_periods = weekly_view.periods;

        Ok(Self {
            daily_work_orders: daily_parameters,
            daily_periods,
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

#[allow(dead_code)]
impl DailyParameters
{
    pub(crate) fn daily_parameter(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&DailyParameter>
    {
        let daily_parameter = self.daily_work_orders
            .get(&work_order_activity.0)
            .context(format!("WorkOrderNumber: {:?} was not part of the DailyParameters", work_order_activity.0))?
            .get(&work_order_activity.1)
            .context(format!("WorkOrderNumber: {:?} with ActivityNumber: {:?} was not part of the DailyParameters", work_order_activity.0, work_order_activity.1))?;

        Ok(daily_parameter)
    }

    // TODO: Consider moving this to the `Parameters` trait
    pub(crate) fn insert_daily_parameter(
        &mut self,
        work_order_activity: &WorkOrderActivity,
        daily_parameter: DailyParameter,
    )
    {
        self.daily_work_orders
            .entry(work_order_activity.0)
            .or_default()
            .insert(work_order_activity.1, daily_parameter);
    }
}

#[derive(Debug, Clone)]
pub struct DailyParameter
{
    pub resource: Skill,
    pub number_of_people: NumberOfPeople,
    pub work_remaining: Work,
}

impl DailyParameter
{
    pub fn new(resource: Skill, number: NumberOfPeople, work_remaining: Work) -> Self
    {
        Self {
            resource,
            number_of_people: number,
            work_remaining,
        }
    }
}

impl Inspect for DailyParameters
{
    fn summary(&self) -> impl fmt::Display + '_
    {
        struct Summary<'a>(&'a DailyParameters);
        impl fmt::Display for Summary<'_>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
            {
                let total_activities: usize =
                    self.0.daily_work_orders.values().map(|inner| inner.len()).sum();
                write!(
                    f,
                    "DailyParameters: {} work orders, {} activities, {} periods",
                    self.0.daily_work_orders.len(),
                    total_activities,
                    self.0.daily_periods.len()
                )
            }
        }
        Summary(self)
    }

    fn state(&self) -> impl fmt::Display + '_
    {
        struct State<'a>(&'a DailyParameters);
        impl fmt::Display for State<'_>
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
            {
                let params = self.0;
                let total_activities: usize =
                    params.daily_work_orders.values().map(|inner| inner.len()).sum();
                let total_work = params
                    .daily_work_orders
                    .values()
                    .flat_map(|inner| inner.values())
                    .fold(Work::from(0.0), |acc, p| acc + p.work_remaining);
                let unique_skills: HashSet<Skill> = params
                    .daily_work_orders
                    .values()
                    .flat_map(|inner| inner.values())
                    .map(|p| p.resource)
                    .collect();

                writeln!(f, "DailyParameters:")?;
                writeln!(
                    f,
                    "  work orders: {}, activities: {}",
                    params.daily_work_orders.len(),
                    total_activities
                )?;
                writeln!(f, "  total work remaining: {}", total_work)?;
                writeln!(f, "  unique skills: {}", unique_skills.len())?;
                write!(
                    f,
                    "  periods: {}",
                    params
                        .daily_periods
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        State(self)
    }
}
