use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::{self};
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::assignments::AnyAssignment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrderPolicies;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use serde::Serialize;

use super::project_resources::ProjectResources;

#[derive(Debug)]
pub struct ProjectParameters
{
    pub project_work_orders: HashMap<WorkOrderNumber, ProjectParameter>,
    pub project_days: Vec<Day>,
    pub project_capacity: ProjectResources,
}

impl Parameters for ProjectParameters
{
    type Key = WorkOrderNumber;

    fn from_scheduling_hypergraph(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let actor_specification = &scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .with_context(|| {
                format!(
                    "Asset: {} is not present in the SchedulingEnvironment",
                    id.asset()
                )
            })?;

        let work_order_policies = &scheduling_environment.work_order_policies;

        let work_orders = scheduling_environment
            .work_orders
            .inner
            .iter()
            // Warning: Every agent should always be connected to an asset.
            .filter(|(_, wo)| &wo.functional_location().asset == id.2.main_asset())
            .filter(|(_, wo)| wo.released_for_scheduling());

        let assignments = &scheduling_environment.assignments.assignment_for_project();
        let project_capacity = ProjectResources::from((scheduling_environment, id));

        let project_work_orders: HashMap<WorkOrderNumber, ProjectParameter> = work_orders
            .map(|(won, wo)| {
                let start_days_for_activities: HashMap<Option<ActivityNumber>, AnyAssignment> =
                    assignments
                        .iter()
                        .filter(|e| e.1.work_order_number() == *won)
                        .map(|e| (e.1.activity_number(), e.1.clone()))
                        .collect::<HashMap<_, _>>();
                Ok((
                    *won,
                    // TODO: Design logic for inverting database constraints
                    create_project_parameter(wo, start_days_for_activities, work_order_policies)?,
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, ProjectParameter>>>()?;

        let project_days = scheduling_environment.time_environment.days[0..min(
            actor_specification.project().number_of_project_days,
            scheduling_environment.time_environment.days.len(),
        )]
            .to_vec();
        Ok(Self {
            project_work_orders,
            project_days,
            project_capacity,
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

// TODO: Consider making `create_parameter` functions associated trait methods
// that accept generic types
pub fn create_project_parameter(
    work_order: &WorkOrder,
    start_days_for_activities: HashMap<Option<ActivityNumber>, AnyAssignment>,
    work_order_configuration: &WorkOrderPolicies,
) -> Result<ProjectParameter>
{
    let mut operation_parameters = BTreeMap::new();
    for activity_number in &work_order.activity_numbers() {
        let forced_day = start_days_for_activities
            .get(&Some(*activity_number))
            .map(|e| e.day())
            .and_then(|e| e);

        let operation = work_order.operation(*activity_number);
        let operation_parameter = OperationParameter::new(
            work_order.work_order_number(),
            operation,
            Work::from(work_order_configuration.operating_time as f64),
            forced_day,
        )?;
        operation_parameters.insert(*activity_number, operation_parameter);
    }

    ProjectParameter::new(work_order, work_order_configuration, operation_parameters)
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectParameter
{
    pub project_operation_parameters: BTreeMap<ActivityNumber, OperationParameter>,
    // TODO: ISSUE #300 Implement forced_schedule_* in the project actor
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: Move earliest_allowed_start_date to SchedulingEnvironment
    pub earliest_allowed_start_date: NaiveDate,
}

impl ProjectParameter
{
    pub fn new(
        work_order: &WorkOrder,
        work_order_configuration: &WorkOrderPolicies,
        operation_parameters: BTreeMap<ActivityNumber, OperationParameter>,
    ) -> Result<Self>
    {
        Ok(Self {
            project_operation_parameters: operation_parameters,
            // TODO: ISSUE #300 Implement forced_schedule_* in the project actor
            weight: work_order.work_order_value(work_order_configuration)?,
            relations: work_order.activity_relations(),
            earliest_allowed_start_date: work_order.earliest_allowed_start_date(),
        })
    }
}

// TODO: Add earliest_start_day field
#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter
{
    pub work_order_number: WorkOrderNumber,
    // TODO: ISSUE #300 Implement forced_schedule_* in the project actor
    // pub forced_start_day: Option<Day>,
    pub number: NumberOfPeople,
    pub duration: Work,
    pub operating_time: Work,
    pub work_remaining: Work,
    pub resource: Skill,
    pub forced_start_date: Option<Day>,
    pub earliest_start_date: DateTime<Utc>,
    pub earliest_finish_date: DateTime<Utc>,
}

impl OperationParameter
{
    pub fn new(
        work_order_number: WorkOrderNumber,
        operation: &Operation,
        operating_time: Work,
        forced: Option<Day>,
    ) -> Result<Self>
    {
        let operation_view = operation.view();
        Ok(Self {
            work_order_number,
            number: operation_view.number_of_people,
            // TODO: Refactor duration initialization
            duration: operation_view.duration,
            operating_time,
            work_remaining: operation_view.remaining_work,
            resource: operation_view.resource,
            earliest_start_date: operation_view.earliest_start_datetime,
            earliest_finish_date: operation_view.earliest_finish_datetime,
            forced_start_date: forced,
        })
    }
}

impl Display for OperationParameter
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "OperationParameters:\n
        {:?}\n
        number: {}\n
        duration: {}\n
        operating_time: {:?}\n
        work_remaining: {}\n
        resource: {}",
            self.work_order_number,
            self.number,
            self.duration,
            self.operating_time,
            self.work_remaining,
            self.resource
        )
    }
}
