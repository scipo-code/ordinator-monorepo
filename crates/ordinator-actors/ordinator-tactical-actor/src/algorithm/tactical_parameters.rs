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
use ordinator_scheduling_environment::worker_environment::TacticalOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;

use super::tactical_resources::TacticalResources;

#[derive(Debug)]
pub struct TacticalParameters
{
    pub tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter>,
    pub tactical_days: Vec<Day>,
    pub tactical_capacity: TacticalResources,
    pub tactical_options: TacticalOptions,
}

impl Parameters for TacticalParameters
{
    type Key = WorkOrderNumber;

    fn from_source(
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
            // WARN: Unwrap accepted. Every agent should always be connected to an Asset
            // QUESTION: Is this actually true?
            .filter(|(_, wo)| &wo.functional_location().asset == id.2.main_asset())
            .filter(|(_, wo)| wo.released_for_scheduling());

        let assignments = &scheduling_environment.assignments.assignment_for_tactical();
        let tactical_capacity = TacticalResources::from((scheduling_environment, id));

        let tactical_work_orders: HashMap<WorkOrderNumber, TacticalParameter> = work_orders
            .map(|(won, wo)| {
                let start_days_for_activities: HashMap<Option<ActivityNumber>, AnyAssignment> =
                    assignments
                        .iter()
                        .filter(|e| e.1.work_order_number() == *won)
                        .map(|e| (e.1.activity_number(), e.1.clone()))
                        .collect::<HashMap<_, _>>();
                Ok((
                    *won,
                    // This should also be found inside of the database.
                    // There is something that has to be inverted here. You are not designing this
                    // is the best possible way.
                    create_tactical_parameter(wo, start_days_for_activities, work_order_policies)?,
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, TacticalParameter>>>()?;

        let tactical_days = scheduling_environment.time_environment.days[0..min(
            actor_specification.tactical().number_of_tactical_days,
            scheduling_environment.time_environment.days.len(),
        )]
            .to_vec();
        Ok(Self {
            tactical_work_orders,
            tactical_days,
            tactical_capacity,
            tactical_options: actor_specification.tactical().tactical_options.clone(),
        })
    }

    // We cannot reuse this component.
    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

// TODO
// We should think carefully about putting this into the `Parameters` trait as
// an associated function. These `create_parameter` functions will be insanely
// important later on. Say every algorithm should have their own of these
// functions... No there should only be one and that one should accept a
// Generic?
//
// Is that even possible? I think that it is. Keep this up! You have to
// continue.
pub fn create_tactical_parameter(
    work_order: &WorkOrder,
    start_days_for_activities: HashMap<Option<ActivityNumber>, AnyAssignment>,
    work_order_configuration: &WorkOrderPolicies,
) -> Result<TacticalParameter>
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

    TacticalParameter::new(work_order, work_order_configuration, operation_parameters)
}

#[derive(Debug, Clone, Serialize)]
pub struct TacticalParameter
{
    pub tactical_operation_parameters: BTreeMap<ActivityNumber, OperationParameter>,
    // ISSUE #300 TODO [ ] 2025-07-17 implement the `forced_schedule_*` in the tactical
    //
    // pub forced_in_period: Option<Period>,
    // You have to make a force schedule function. You are getting a
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: These two should be moved out of the parameters. You might end up implementing
    // that on the these on the `SchedulingEnvironment`
    pub earliest_allowed_start_date: NaiveDate,
}

// How should the parameters be build here?
impl TacticalParameter
{
    pub fn new(
        work_order: &WorkOrder,
        // This should be a part of the options.
        work_order_configuration: &WorkOrderPolicies,
        operation_parameters: BTreeMap<ActivityNumber, OperationParameter>,
    ) -> Result<Self>
    {
        Ok(Self {
            tactical_operation_parameters: operation_parameters,
            // Setting this field will be immensely difficult. This is where much of the business
            // logic should recide. It will not be simple to create this.
            // This is a crazy place to code the system. You have to work more on this.
            // Should you wait with this? Yes you should... You cannot see the path forward
            // and it is causing you pain here. The best approach forward here is to make the
            // system function with o
            // ISSUE #300 TODO [ ] 2025-07-17 implement the `forced_schedule_*` in the tactical
            // actor forced_in_period: work_order.,
            weight: work_order.work_order_value(work_order_configuration)?,
            relations: work_order.activity_relations(),
            earliest_allowed_start_date: work_order.earliest_allowed_start_date(),
            // So we have to create something that will let us fix the tactical work_orders. The
            // enum here is a great bet.
        })
    }
}

// You cannot make a forced method for the `tactical` as you do not know where
// to fit them in this.
// TODO [x] This is lacking the earliest_strat_day. That is not good that
// it is missing.
#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter
{
    // Okay here you need to add additional logic to the system. The most important thing is to
    // make the system work correctly with the
    pub work_order_number: WorkOrderNumber,
    // ISSUE #300 TODO [ ] 2025-07-17 implement the `forced_schedule_*` in the tactical
    // pub forced_start_day: Option<Day>,
    pub number: NumberOfPeople,
    pub duration: Work,
    pub operating_time: Work,
    pub work_remaining: Work,
    pub resource: Resources,
    pub forced_start_date: Option<Day>,
    // You did something right here. What was that? I am not really sure here.
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
            // FIX
            // This should also have been created differently.
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
