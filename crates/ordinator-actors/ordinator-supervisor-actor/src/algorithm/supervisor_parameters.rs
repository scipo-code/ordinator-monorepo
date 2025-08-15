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
use ordinator_scheduling_environment::worker_environment::resources::Resources;

pub struct SupervisorParameters
{
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
    pub options: SupervisorOptions,
}

pub struct SupervisorParametersBuilder;
// ASSERT on elements in the Vec. That is a really good point.
// ISSUE START HERE
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
            // You should really use the
            // NOTE [ ] CRUCIAL LESSON
            // You should use a debugger. I think that you should
            // You head is a faulty interpreter! Good point! I think that
            // we should.
            panic!("Use the alternate version of the Debug formatter")
        }
    }
}

impl Parameters for SupervisorParameters
{
    type Builder = SupervisorParametersBuilder;
    type Key = WorkOrderActivity;

    fn from_scheduling_environment(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let mut supervisor_parameters = HashMap::new();

        // Should you Clone this? Yes.. But ideally you should simply use Functional
        // programming. That is the only way in a situation like this.
        // You should make part of the SchedulingEnvironment reside inside of the
        // Arc<WorkOrders> and the other part an ArcSwap<TimeEnvironment>
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
            // ISSUE #130
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

    fn from_builder() -> Self::Builder
    {
        todo!()
    }
}

#[allow(dead_code)]
impl SupervisorParameters
{
    // ISSUE #000
    // make-the-actor-create-parameters-directly-from-the-scheduling-environment
    //
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

    // This should be a part of the `Parameters` trait. You are starting to feel
    // overwhelmed again. Relax
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
        // DEBUG: Make assertions here!
    }
}

#[derive(Debug, Clone)]
pub struct SupervisorParameter
{
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
}

impl SupervisorParameter
{
    pub fn new(resource: Resources, number: NumberOfPeople, work_remaining: Work) -> Self
    {
        Self {
            resource,
            number,
            work_remaining,
        }
    }
}
