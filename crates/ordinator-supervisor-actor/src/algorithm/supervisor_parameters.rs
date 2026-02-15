use std::collections::HashMap;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::SupervisorOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;

pub struct SupervisorParameters
{
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
    pub options: SupervisorOptions,
}

// TODO: Add assertions on vector elements
impl std::fmt::Debug for SupervisorParameters
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            write!(
                f,
                "Number of WorkOrderActivities: {}\n\
                Supervisor periods: {:#?}",
                self.supervisor_work_orders.len(),
                self.supervisor_periods,
            )
        } else {
            panic!("Use the alternate version of the Debug formatter")
        }
    }
}

impl Parameters for SupervisorParameters
{
    type Key = WorkOrderActivity;

    fn from_source(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let mut supervisor_parameters = HashMap::new();

        // Consider refactoring SchedulingEnvironment to use Arc<WorkOrders> and ArcSwap<TimeEnvironment>
        // for better performance with Functional programming patterns
        let input_supervisor = scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .unwrap()
            .supervisor()
            .iter()
            .find(|e| e.id == *id.0)
            .with_context(|| format!("Missing an Supervisor entry for {id}"))?;

        let options = input_supervisor
            .supervisor_options
            .clone();

        let supervisor_periods = &scheduling_environment
            .time_environment
            .periods
            .get(0..input_supervisor.number_of_supervisor_periods as usize)
            .with_context(||format!("There are not enough periods in the TimeEnvironment to initialize the Supervisor\nNumber of supervisor periods: {}", input_supervisor.number_of_supervisor_periods))?;

        for (work_order_number, work_order) in scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(_, wo)| &wo.functional_location().asset == id.2.main_asset())
        {
            let mut inner_map = HashMap::new();
            for activity_number in work_order.activity_numbers() {
                let resource = work_order.operation_resource(activity_number)?;
                let number = work_order.number_of_people(activity_number)?;
                let work = work_order.operation_work_remaining(activity_number)?;

                let supervisor_parameter = SupervisorParameter::new(resource, number, work);

                inner_map.insert(activity_number, supervisor_parameter);
            }

            let _assert_option = supervisor_parameters.insert(*work_order_number, inner_map);

            assert!(_assert_option.is_none());
        }

        Ok(Self {
            supervisor_work_orders: supervisor_parameters,
            supervisor_periods: supervisor_periods.to_vec(),
            options,
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
impl SupervisorParameters
{
    pub(crate) fn supervisor_parameter(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&SupervisorParameter>
    {
        let supervisor_parameter = self.supervisor_work_orders
            .get(&work_order_activity.0)
            .context(format!("WorkOrderNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0))?
            .get(&work_order_activity.1)
            .context(format!("WorkOrderNumber: {:?} with ActivityNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0, work_order_activity.1))?;

        Ok(supervisor_parameter)
    }

    // TODO: Consider moving this to the `Parameters` trait
    pub(crate) fn insert_supervisor_parameter(
        &mut self,
        work_order_activity: &WorkOrderActivity,
        supervisor_parameter: SupervisorParameter,
    )
    {
        self.supervisor_work_orders
            .entry(work_order_activity.0)
            .or_default()
            .insert(work_order_activity.1, supervisor_parameter);
    }
}

#[derive(Debug, Clone)]
pub struct SupervisorParameter
{
    pub resource: Skill,
    pub number_of_people: NumberOfPeople,
    pub work_remaining: Work,
}

impl SupervisorParameter
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
