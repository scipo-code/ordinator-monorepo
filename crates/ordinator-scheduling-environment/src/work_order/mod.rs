pub mod display;
pub mod operation;
pub mod work_order_analytic;
pub mod work_order_dates;
pub mod work_order_info;
pub mod work_orders_api;

use std::collections::HashMap;
use std::collections::HashSet;
use std::num::ParseIntError;
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::TimeDelta;
use chrono::Utc;
use colored::Colorize;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use work_order_analytic::status_codes::SystemStatusCodes;
use work_order_analytic::status_codes::UserStatusCodes;
use work_order_dates::WorkOrderDatesBuilder;

use self::operation::ActivityNumber;
use self::operation::Operation;
use self::operation::OperationBuilder;
use self::operation::Operations;
use self::operation::Work;
use self::work_order_analytic::WorkOrderAnalytic;
use self::work_order_analytic::WorkOrderAnalyticBuilder;
use self::work_order_analytic::status_codes::MaterialStatus;
use self::work_order_dates::WorkOrderDates;
use self::work_order_info::WorkOrderInfo;
use self::work_order_info::WorkOrderInfoBuilder;
use self::work_order_info::functional_location::FunctionalLocation;
use self::work_order_info::priority::Priority;
use self::work_order_info::work_order_type::WorkOrderType;
use super::time_environment::period::Period;
use super::worker_environment::resources::Skill;
use crate::Asset;
use crate::materials::MaterialToPeriod;
use crate::time_environment::day::Day;
use crate::worker_environment::IdString;
use crate::worker_environment::resources::ActorCompositeId;

// TODO [ ]
//
// You should really consider making this into a struct so that
// you can define custom behavior on it. I do not think that there
// is a better way of defining the code here.
pub type WorkOrderActivity = (WorkOrderNumber, ActivityNumber);
pub type WorkOrderValue = u64;

#[derive(Copy, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct WorkOrderNumber(pub u64);
impl WorkOrderNumber
{
    pub fn is_dummy(&self) -> bool
    {
        self.0 == 0
    }
}

impl std::fmt::Debug for WorkOrderNumber
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{}",
            format!("WorkOrderNumber({})", self.0).bright_yellow()
        )
    }
}

impl std::fmt::Display for WorkOrderNumber
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}
// Everything in the `SchedulingEnvironment` should implement
// `Serialize` it has to, to be able to go into the database.
type SubnetworkNumber = u64;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrders
{
    pub inner: HashMap<WorkOrderNumber, WorkOrder>,
    // TODO [ ] 2025-07-22 extend the `WorkOrders` to contain
    // `Subnetwork`
    _subnetwork: HashMap<SubnetworkNumber, Vec<WorkOrder>>,
    // Are these in the correct place in the code? Yes I think
    // that they are.
}

impl WorkOrders
{
    pub fn update_material_checked(
        &mut self,
        work_order_number: &WorkOrderNumber,
        checked: bool,
    ) -> anyhow::Result<()>
    {
        self.inner
            .get_mut(work_order_number)
            .with_context(|| {
                format!(
                    "work order {:#?} not present in WorkOrder",
                    work_order_number
                )
            })?
            .material_checked = checked;

        Ok(())
    }

    pub fn resource(&self, work_order_activity: &(WorkOrderNumber, u64)) -> Option<Skill>
    {
        Some(
            self.inner
                .get(&work_order_activity.0)?
                .operations
                .0
                .get(&work_order_activity.1)?
                .resource,
        )
    }
}

// What should you do now?
// WARN
// Configurations should only be used during initialization not the
// remaining parts of the code.
pub struct WorkOrdersBuilder
{
    inner: Option<HashMap<WorkOrderNumber, WorkOrder>>,
    _subnetwork: Option<HashMap<SubnetworkNumber, Vec<WorkOrder>>>,
}

impl WorkOrdersBuilder
{
    pub fn build(self) -> WorkOrders
    {
        WorkOrders {
            inner: self.inner.unwrap_or_default(),
            _subnetwork: HashMap::default(),
        }
    }

    pub fn work_order_builder<F>(mut self, work_order_number: WorkOrderNumber, f: F) -> Self
    where
        F: FnOnce(WorkOrderBuilder) -> WorkOrderBuilder,
    {
        let work_order_builder = WorkOrder::builder(work_order_number);

        let work_order_builder = f(work_order_builder);

        match &mut self.inner {
            Some(work_orders_inner) => {
                work_orders_inner.insert(
                    work_order_builder.work_order_number,
                    work_order_builder.build(),
                );
            }
            None => {
                let work_order_inner = HashMap::from([(
                    work_order_builder.work_order_number,
                    work_order_builder.build(),
                )]);

                self.inner = Some(work_order_inner);
            }
        }
        self
    }

    pub fn work_orders_manual(self, work_orders_inner: HashMap<WorkOrderNumber, WorkOrder>)
    -> Self
    {
        Self {
            inner: Some(work_orders_inner),
            _subnetwork: None,
        }
    }

    pub fn subnetwork_manual(self, subnetwork: HashMap<SubnetworkNumber, Vec<WorkOrder>>) -> Self
    {
        Self {
            inner: self.inner,
            _subnetwork: Some(subnetwork),
        }
    }
}

impl WorkOrders
{
    pub fn builder() -> WorkOrdersBuilder
    {
        WorkOrdersBuilder {
            inner: Some(HashMap::new()),
            _subnetwork: None,
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder)
    {
        self.inner.insert(work_order.work_order_number, work_order);
    }

    pub fn new_work_order(&self, work_order_number: WorkOrderNumber) -> bool
    {
        !self.inner.contains_key(&work_order_number)
    }

    pub fn work_orders_by_asset(&self, asset: &Asset) -> HashMap<&WorkOrderNumber, &WorkOrder>
    {
        self.inner
            .iter()
            .filter(|(_, wo)| &wo.work_order_info.functional_location.asset == asset)
            .collect()
    }
}

// TODO [ ] 2025-07-22 make a common locked functionality state
// Also the `Scheduled` should be modelled in a different way than
// using the status codes. The `WorkOrder` should flow through the
// system towards the correct state. I think that using the `TypeState`
// pattern here would be a good idea.
//
// ESSAY:
// How should the code look here? Should you make a type? You
// have originally thought that this should be handled in the
// `Actors` but that is not the correct approach here. You
// should strive to centralize the state required here.
//
// You should make a `method`! Great! That is the best path
// forward here! You do not want to add any thing here.
//
// You need a single method returning the correct state!
// And it should be defined on the `WorkOrder` what about
// the `Subnetwork`
//
// ESSAY: WorkOrder and Subnetwork interaction.
// This is a crucial point for later on. I bacically
// means that there can be interactions between the
// different WorkOrders. And this in turn means that
// there can be interactions between different days
// of the work order. The best approach for solving
// this is probably to make the system work correctly.
//
// Remember that the `basic_start` migth have to
// be a `Arc<Mutex<DateTime<Utc>>` so that when
// you move a work order around all the different
// days are changed!
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrder
{
    pub(crate) work_order_number: WorkOrderNumber,
    pub(crate) main_work_center: Skill,
    pub(crate) operations: Operations,
    pub(crate) material_checked: bool,
    pub(crate) work_order_analytic: WorkOrderAnalytic,
    pub(crate) work_order_dates: WorkOrderDates,
    pub(crate) work_order_info: WorkOrderInfo,
    // This is not acceptable. You should move it out
    pub(crate) fixed_by: FixedWorkOrder,
}

/// [`WorkOrder`] read methods.
impl WorkOrder
{
    pub fn earliest_allowed_start_date(&self) -> NaiveDate
    {
        self.work_order_dates.earliest_allowed_start_date
    }

    pub fn operation_work_remaining(&self, activity_number: u64) -> Result<Work>
    {
        Ok(self
            .operations
            .0
            .get(&activity_number)
            .context("Operation does not exist")?
            .operation_info
            .work_remaining)
    }

    pub fn operation_preparation(&self, activity: u64) -> Option<Work>
    {
        Some(
            self.operations
                .0
                .get(&activity)?
                .operation_analytic
                .preparation_time,
        )
    }

    pub fn activity_numbers(&self) -> Vec<u64>
    {
        self.operations.0.keys().cloned().collect()
    }

    pub fn number_of_people(&self, activity_number: u64) -> Result<u64>
    {
        Ok(self
            .operations
            .0
            .get(&activity_number)
            .context("Operation missing")?
            .operation_info
            .number)
    }

    pub fn operation_resource(&self, activity_number: u64) -> Result<Skill>
    {
        Ok(self
            .operations
            .0
            .get(&activity_number)
            .context("Operation missing")?
            .resource)
    }

    pub fn operation(&self, activity_number: u64) -> &Operation
    {
        self.operations
            .0
            .get(&activity_number)
            .expect("Should should never be missing")
    }

    pub fn work_order_number(&self) -> WorkOrderNumber
    {
        self.work_order_number
    }
}

// This is not allowed.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FixedWorkOrder
{
    // You are mixing workers and `time`
    Period(Period),
    BasicStart(NaiveDate),
    Operational(DateTime<Utc>),
    BusinessLogic,
}

// Should you work on this now? No! Practice skills and read.
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct ManuallyInputtedInformation
// {
//     excluded_periods: HashSet<Rc<Period>>,
// }

pub struct WorkOrderBuilder
{
    work_order_number: WorkOrderNumber,
    main_work_center: Option<Skill>,
    operations: Operations,
    // FIX
    // Every operation needs to have a relation between them. There
    // is no way around this. It should be an enforced invariant.
    work_order_analytic: Option<WorkOrderAnalytic>,
    work_order_dates: Option<WorkOrderDates>,
    work_order_info: Option<WorkOrderInfo>,
    _fixed_by: Option<FixedWorkOrder>,
}

impl WorkOrderBuilder
{
    pub fn build(self) -> WorkOrder
    {
        WorkOrder {
            work_order_number: self.work_order_number,
            main_work_center: self
                .main_work_center
                .expect("Missing field initializations on the WorkOrderBuilder"),
            operations: self.operations,
            work_order_analytic: self
                .work_order_analytic
                .expect("Missing field initializations on the WorkOrderBuilder"),
            work_order_dates: self
                .work_order_dates
                .expect("Missing field initializations on the WorkOrderBuilder"),
            work_order_info: self
                .work_order_info
                .expect("Missing field initializations on the WorkOrderBuilder"),
            fixed_by: FixedWorkOrder::BusinessLogic,
            material_checked: false,
        }
    }

    pub fn work_order_number(mut self, work_order_number: WorkOrderNumber) -> Self
    {
        self.work_order_number = work_order_number;
        self
    }

    pub fn main_work_center(mut self, main_work_center: Skill) -> Self
    {
        self.main_work_center = Some(main_work_center);
        self
    }

    // TODO [ ]
    // Make this function simply reuse the functionality of the `Operations`.
    // QUESTION
    // How do we do this?
    // This is crucial! There is something that you do not understand here
    // How do we extract this so that it works?
    pub fn operations_builder<F>(
        mut self,
        operation_number: u64,
        resource: Skill,
        f: F,
    ) -> Result<Self>
    where
        F: FnOnce(OperationBuilder) -> OperationBuilder,
    {
        let operations_builder = Operation::builder(operation_number, resource);

        let operations_builder = f(operations_builder);

        if let Some(operation) = self
            .operations
            .0
            .insert(operation_number, operations_builder.build())
        {
            bail!("Activity: {} is already built", operation.activity)
        }

        Ok(self)
    }

    pub fn operations(mut self, operations: Operations) -> Self
    {
        self.operations = operations;
        self
    }

    pub fn work_order_analytic_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderAnalyticBuilder) -> WorkOrderAnalyticBuilder,
    {
        let work_order_analytic_builder = WorkOrderAnalytic::builder();

        let work_order_analytic_builder = configure(work_order_analytic_builder);

        self.work_order_analytic = Some(work_order_analytic_builder.build());
        self
    }

    pub fn work_order_info_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderInfoBuilder) -> WorkOrderInfoBuilder,
    {
        let work_order_info_builder = WorkOrderInfo::builder();

        let work_order_info_builder = configure(work_order_info_builder);

        self.work_order_info = Some(work_order_info_builder.build());
        self
    }

    pub fn work_order_dates_builder<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(WorkOrderDatesBuilder) -> WorkOrderDatesBuilder,
    {
        let work_order_dates_builder = WorkOrderDates::builder();

        let work_order_dates_builder = configure(work_order_dates_builder);

        self.work_order_dates = Some(work_order_dates_builder.build());
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityRelation
{
    StartStart,
    FinishStart,
    Postpone(TimeDelta),
}

// `operating_time` is separate from the work order data and should be removed
// from the `scheduling_environment`
#[allow(dead_code)]
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPolicies
{
    pub order_type_weights: HashMap<String, u64>,
    pub status_weights: HashMap<String, u64>,
    pub vis_priority_map: HashMap<char, u64>,
    pub wdf_priority_map: HashMap<String, u64>,
    pub wgn_priority_map: HashMap<String, u64>,
    pub wpm_priority_map: HashMap<char, u64>,
    pub clustering_weights: ClusteringWeights,
    pub operating_time: u64,
}

pub struct WorkOrderPoliciesBuilder
{
    order_type_weights: Option<HashMap<String, u64>>,
    status_weights: Option<HashMap<String, u64>>,
    vis_priority_map: Option<HashMap<char, u64>>,
    wdf_priority_map: Option<HashMap<String, u64>>,
    wgn_priority_map: Option<HashMap<String, u64>>,
    wpm_priority_map: Option<HashMap<char, u64>>,
    clustering_weights: Option<ClusteringWeights>,
    operating_time: Option<u64>,
}

impl WorkOrderPoliciesBuilder
{
    pub fn new() -> Self
    {
        Self {
            order_type_weights: None,
            status_weights: None,
            vis_priority_map: None,
            wdf_priority_map: None,
            wgn_priority_map: None,
            wpm_priority_map: None,
            clustering_weights: None,
            operating_time: None,
        }
    }

    pub fn order_type_weights(mut self, order_type_weights: HashMap<String, u64>) -> Self
    {
        self.order_type_weights = Some(order_type_weights);
        self
    }

    pub fn status_weights(mut self, status_weights: HashMap<String, u64>) -> Self
    {
        self.status_weights = Some(status_weights);
        self
    }

    pub fn vis_priority_map(mut self, vis_priority_map: HashMap<char, u64>) -> Self
    {
        self.vis_priority_map = Some(vis_priority_map);
        self
    }

    pub fn wdf_priority_map(mut self, wdf_priority_map: HashMap<String, u64>) -> Self
    {
        self.wdf_priority_map = Some(wdf_priority_map);
        self
    }

    pub fn wgn_priority_map(mut self, wgn_priority_map: HashMap<String, u64>) -> Self
    {
        self.wgn_priority_map = Some(wgn_priority_map);
        self
    }

    pub fn wpm_priority_map(mut self, wpm_priority_map: HashMap<char, u64>) -> Self
    {
        self.wpm_priority_map = Some(wpm_priority_map);
        self
    }

    pub fn clustering_weights(mut self, clustering_weights: ClusteringWeights) -> Self
    {
        self.clustering_weights = Some(clustering_weights);
        self
    }

    pub fn operating_time(mut self, operating_time: u64) -> Self
    {
        self.operating_time = Some(operating_time);
        self
    }

    pub fn build(self) -> WorkOrderPolicies
    {
        WorkOrderPolicies {
            order_type_weights: self.order_type_weights.unwrap_or_default(),
            status_weights: self.status_weights.unwrap_or_default(),
            vis_priority_map: self.vis_priority_map.unwrap_or_default(),
            wdf_priority_map: self.wdf_priority_map.unwrap_or_default(),
            wgn_priority_map: self.wgn_priority_map.unwrap_or_default(),
            wpm_priority_map: self.wpm_priority_map.unwrap_or_default(),
            clustering_weights: self.clustering_weights.unwrap_or_default(),
            operating_time: self.operating_time.unwrap_or(6),
        }
    }
}

impl WorkOrderPolicies
{
    pub fn builder() -> WorkOrderPoliciesBuilder
    {
        WorkOrderPoliciesBuilder::new()
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClusteringWeights
{
    pub asset: u64,
    pub sector: u64,
    pub system: u64,
    pub subsystem: u64,
    pub equipment_tag: u64,
}

// ESSAY: Should this type handle both forced and excluded?
// I think that is a good idea. Maybe a better option would
// be to make two types,
//
// ESSAY: In operation research is there really only:
// 1. The required decisions
// 2. The not allowed decisions
// 3. The open solution space of the decisions that can be optimized
// these three types?
//
// Effectively they are all constraints. For the network flow problems
// there is the the flows that needs to be satisfied, the paths that cannot
// be taken, and then there is the decisions that are up for the model to
// decide.
//
// For the graph coloring problem. All vertices needs a color (required), two
// vercies cannot share an edge and have the same color (not allowed), find the
// least amount of colors.
// Does this cover everything that the code needs to do here? You will need
// to add a rules engine at somepoint, and you will need the hexagonal
// architecture from the start.
#[derive(Debug)]
pub enum ForcedWorkOrder
{
    Period((Period, HashSet<Period>)),
    Days(TacticalForceType),
    Technician(TechnicianInclude, TechnicianExclude),
    FreeWorkOrder,
}

#[derive(Debug)]
pub enum TacticalForceType
{
    OnlyStartDay(Day),
    IndividualActivities(Vec<Day>, Vec<HashSet<Day>>),
}
impl TacticalForceType
{
    pub fn get_contained_date(&self) -> NaiveDate
    {
        match self {
            TacticalForceType::OnlyStartDay(day) => day.date,
            TacticalForceType::IndividualActivities(start_days_per_activity, _) => {
                start_days_per_activity
                    .first()
                    .expect("A Day should always be contained in a period.")
                    .date
            }
        }
    }

    pub fn excluded_days(&self, per: &Period) -> bool
    {
        match self {
            TacticalForceType::OnlyStartDay(_day) => false,
            TacticalForceType::IndividualActivities(_, excluded_days) => excluded_days
                .iter()
                .flatten()
                .all(|f| per.day_indices.contains(&(f.day_index as u64))),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TechnicianInclude
{
    pub id: IdString,
    pub interval: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TechnicianExclude
{
    ids: HashSet<ActorCompositeId>,
    intervals: HashSet<Option<(Day, Day)>>,
}

// pub enum SchedulePlan
// {
//     ForcedPeriod,
//     ForcedDays,
//     ForcedTechnician,
//     // Remember that the only thing that you want here is
//     // 1. Required Decisions (Include)
//     // 2. Disallowed Decisions (Exclude)
//     // 3. Optimizable Desisions (Weight and optimize)
//     //
//     // Rules have to determine the first and the second, weight have to
//     // determine the third. Yes I think that we have the approach going
// forward. }

// pub struct ScheduleContribution
// {
//     vendor: bool,
//     sch: bool,
//     aswc: bool,
// }

// pub trait ScheduleRule
// {
//     fn is_applicable(&self, work_order: &WorkOrder) -> bool;
//     fn contribution(&self, work_order: &WorkOrder) -> ScheduleContribution;
// }

// Is this a good idea? No
// struct Vendor;

// impl ScheduleRule for Vendor
// {
//     fn is_applicable(&self, _work_order: &WorkOrder) -> bool
//     {
//         todo!()
//     }

//     fn contribution(&self, _work_order: &WorkOrder) -> ScheduleContribution
//     {
//         todo!()
//     }
// }

impl WorkOrder
{
    pub fn builder(work_order_number: WorkOrderNumber) -> WorkOrderBuilder
    {
        WorkOrderBuilder {
            work_order_number,
            main_work_center: None,
            operations: Operations::default(),
            work_order_analytic: None,
            work_order_dates: None,
            work_order_info: None,
            _fixed_by: None,
        }
    }

    pub fn is_valid(&self) -> bool
    {
        false
    }

    pub fn activity_relations(&self) -> Vec<ActivityRelation>
    {
        self.operations.relations()
    }

    pub fn released_for_scheduling(&self) -> bool
    {
        self.work_order_analytic.system_status_codes.rel
    }

    // How should this function be implemented? You need to look in the
    // WorkOrderDates and the Operations and the StatusCodes. The idea
    // here is that you should move the logic out of the `Actor`s and
    // into the `SchedulingEnvironment` is this correct? Yes! You need
    // a single source of truth here.
    //
    // For now simply make it simple. That is the most important! The
    // crucial part here is scheduling the work order so that the
    // system will scale. The logic cannot be in the parameters of the
    // Actors that does simply not scale well enough.
    // QUESTION: Should you make this a free function? Yes I actually think
    // so. You are relying too much on `self` and it makes the dependencies
    // of every method insanely big.
    //
    // You need to think very carefully about constructing this type. It is
    // crucial that the code is each to understand. You will also need to
    // have additional fields in the `SchedulingEnvironment` to accomdate the
    // new informations.
    //
    // 0. Use value objects
    // 0. Use a single constructor
    // 1. Centralize first
    // 2. List new options
    // 3. Choose approach forward
    // 4. Make enum
    // 5. Make applicable rules
    // 6. Implement typestate pattern
    // 7. Implement runtime rules engine
    pub fn forced_work_order(
        &self,
        periods: &[Period],
        days: &[Day],
        material_to_periods: &MaterialToPeriod,
    ) -> Result<ForcedWorkOrder>
    {
        // Okay so the working idea is that every actor should call this. The actor
        // might need certain other informations

        // Ideally we should split the work orders by the operations in the code. I
        // think that is the best approach going forward. For now simply take
        // the first element of the code.
        let unloading_point_period = self
            .operations
            .0
            // This is a whole new nightmare
            .iter()
            .nth(0)
            .unwrap()
            .1
            .unloading_point(periods);

        // So the whole thing is now about returning a single object that works for
        // every Actor that means that we should focus on returning the simplest
        // object possible. That encompass everything.
        //
        // This type
        // The obstacle is the way here.
        match &self.fixed_by {
            FixedWorkOrder::Period(_period) => (),
            FixedWorkOrder::BasicStart(naive_date) => {
                // TODO [ ] include
                let day = days
                    .iter()
                    .find(|f| f.date == *naive_date)
                    .context("naive_date not found in TimeEnvironment")?
                    .clone();
                // You need more complex logic to fix more of the code here.
                // What you are doing here is wrong. You should fix it in a
                // different way.
                return Ok(ForcedWorkOrder::Days(TacticalForceType::OnlyStartDay(day)));
            }
            FixedWorkOrder::Operational(_date_time) => (),
            FixedWorkOrder::BusinessLogic => (),
        }
        if self.vendor()
            && (unloading_point_period.is_some() || self.work_order_analytic.user_status_codes.awsc)
        {
            // This needs to be corrected as well.
            let forced_work_order = match unloading_point_period {
                Some(unloading_point_period) => {
                    let mut excluded_periods =
                        self.find_excluded_periods(periods, material_to_periods);

                    // You have to determine a `Vec<Day>` for each of the workorders.
                    excluded_periods.remove(unloading_point_period);
                    // Okay first make the type, then extend it
                    info!(target: "business_events", work_order_number = self.work_order_number.0, "vendor work order forced in UnloadingPoint");

                    // You should use the basic start dates here.
                    Some(ForcedWorkOrder::Period((
                        unloading_point_period.clone(),
                        excluded_periods,
                    )))
                }
                None => {
                    let scheduled_period = periods
                        .iter()
                        .find(|period| period.contains_date(self.work_order_dates.basic_start_date))
                        .cloned();

                    if let Some(locked_in_period) = scheduled_period {
                        let mut excluded_periods =
                            self.find_excluded_periods(periods, material_to_periods);
                        excluded_periods.remove(&locked_in_period);
                        // Okay first make the type, then extend it
                        info!(target: "business_events", work_order_number = self.work_order_number.0, "vendor work order forced in BasicStartDate");
                        Some(ForcedWorkOrder::Period((
                            locked_in_period.clone(),
                            excluded_periods,
                        )))
                    } else {
                        None
                    }
                }
            };
            // CRUCIAL INSIGHT! You were not using value objects and the newtype pattern.
            // This is a mistake.
            if let Some(forced_work_order) = forced_work_order {
                return Ok(forced_work_order);
            }
        }

        // This should be removed. We cannot determine if this is true or not.
        // if self.vendor() {
        //     self.locked_in_period = None;
        //     self.excluded_periods
        //         .remove(self.locked_in_period.as_ref().unwrap());
        //     info!(target: "business_events", work_order_number =
        // work_order.work_order_number.0, "vendor work order unscheduled (Make sure
        // that this is what you actually want)");     return Ok(self);
        // };

        // Okay make it work first.
        // RULE 5: Only change one thing at a time.
        if self.work_order_analytic.user_status_codes.sch {
            // TODO [ ]
            // This really ought to be made in a completely different way here.
            let forced_work_order = if let Some(unloading_point_period) = unloading_point_period {
                if periods[0..=1].contains(unloading_point_period) {
                    let mut excluded_periods =
                        self.find_excluded_periods(periods, material_to_periods);
                    excluded_periods.remove(unloading_point_period);
                    info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled with SCH and unloading point period");
                    Some(ForcedWorkOrder::Period((
                        unloading_point_period.clone(),
                        excluded_periods,
                    )))
                } else {
                    let scheduled_period = periods[0..=1].iter().find(|period| {
                        period.contains_date(self.work_order_dates.basic_start_date)
                    });

                    if let Some(locked_in_period) = scheduled_period {
                        let mut excluded_periods =
                            self.find_excluded_periods(periods, material_to_periods);
                        excluded_periods.remove(&unloading_point_period.clone());
                        info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled with SCH and BasicStartDate");
                        Some(ForcedWorkOrder::Period((
                            locked_in_period.clone(),
                            excluded_periods,
                        )))
                    } else {
                        None
                    }
                }
            } else {
                None
            };
            if let Some(forced_work_order) = forced_work_order {
                return Ok(forced_work_order);
            }
        }

        // This is the kind of thing that we really want to avoid.
        // Okay we should also move the `basic_start_date`
        if self.work_order_analytic.user_status_codes.awsc {
            let scheduled_period = periods
                .iter()
                .find(|period| period.contains_date(self.work_order_dates.basic_start_date));

            let forced_work_order = if let Some(locked_in_period) = scheduled_period {
                let mut excluded_periods = self.find_excluded_periods(periods, material_to_periods);
                excluded_periods.remove(&locked_in_period.clone());
                info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled with SCH and BasicStartDate");
                Some(ForcedWorkOrder::Period((
                    locked_in_period.clone(),
                    excluded_periods,
                )))
            } else {
                None
            };
            info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled with AWSC and BasicStartDate");
            if let Some(forced_work_order) = forced_work_order {
                return Ok(forced_work_order);
            }
        }

        if self
            .operations
            .0
            .iter()
            .nth(0)
            .unwrap()
            .1
            .unloading_point(periods)
            .is_some()
        {
            let locked_in_period = unloading_point_period.unwrap();
            let forced_work_order = if !periods[0..=1]
                .contains(unloading_point_period.as_ref().unwrap())
            {
                let mut excluded_periods = self.find_excluded_periods(periods, material_to_periods);
                excluded_periods.remove(&locked_in_period.clone());
                info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled with SCH and BasicStartDate");
                Some(ForcedWorkOrder::Period((
                    locked_in_period.clone(),
                    excluded_periods,
                )))
            } else {
                None
            };
            info!(target: "business_events", work_order_number = self.work_order_number.0, "normal work order scheduled unloading point exclusively");
            if let Some(forced_work_order) = forced_work_order {
                return Ok(forced_work_order);
            }
        }

        // QUESTION: What are the factors that can force a `WorkOrder`?
        // 1. UnloadingPoint
        // 2. AWSC
        // 3. SCH
        // 4. BasicStart
        // 5. EASD & LAFD
        // if self.work_order_analytic.awaiting_scheduling() {
        //     let basic_start = self.work_order_dates.basic_start_date;
        //     return Ok(ForcedWorkOrder::Days(basic_start));
        // }

        // Is this correct? I am not really sure here. What should you do
        // if the `UnloadingPoint` and other does not match?
        // if self.work_order_analytic.scheduled() {
        //     let basic_start = self.work_order_dates.basic_start_date;
        //     return Ok(ForcedWorkOrder::Days(basic_start));
        // }

        // TODO [ ] 2025-07-22 make the code work with the ForcedWorkORder::Technician
        // You ideally need to split every operation so that they can
        // be planned whenever.
        // if let Some(period_string) = &self
        //     .operations
        //     .0
        //     .first_key_value()
        //     .context("Every WorkOrder should have atleast one Operation")?
        //     .1
        //     .unloading_point
        //     .period_string
        // {
        //     let period = periods
        //         .iter()
        //         .find(|p| p.period_string() == *period_string)
        //         .context("`periods: &[Period]` does not contain the
        // `period_string`")?;

        //     return Ok(ForcedWorkOrder::Period(period.clone()));
        // }
        Ok(ForcedWorkOrder::FreeWorkOrder)
    }

    pub fn vendor(&self) -> bool
    {
        self.operations
            .0
            .values()
            .any(|opr| opr.resource.is_ven_variant())
    }

    //TODO [ ] 2025-07-24 a lot of invariants have to be enforced on this.
    pub fn set_basic_start_date(&mut self, basic_start_date: NaiveDate)
    {
        self.work_order_dates.basic_start_date = basic_start_date;
        self.fixed_by = FixedWorkOrder::BasicStart(self.work_order_dates.basic_start_date);
    }

    pub fn work_order_value(
        &self,
        work_order_configurations: &WorkOrderPolicies,
    ) -> Result<WorkOrderValue>
    {
        // FIX
        // This should be removed. Where should the global configs be read
        // from? I am not really sure. You have done a lot today! I think that
        // reading more. Is a really good idea. Maybe finish the one you started
        // quickly.
        // TODO [ ]
        // There can be no stray `configs` like these! They have to be handled
        // in a higher level.
        let base_value = match &self.work_order_info.work_order_type {
            WorkOrderType::Wdf(wdf_priority) => match wdf_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wdf_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WDF"]
                }
                _ => bail!("Received a wrong input number work order priority: {wdf_priority:#?}"),
            },
            WorkOrderType::Wgn(wgn_priority) => match wgn_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wgn_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WGN"]
                }
                _ => bail!("Received a wrong input number work order priority: {wgn_priority:#?}"),
            },
            WorkOrderType::Wpm(wpm_priority) => match wpm_priority {
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    work_order_configurations.wpm_priority_map[char]
                        * work_order_configurations.order_type_weights["WPM"]
                }
                _ => bail!("Received a wrong input number work order priority: {wpm_priority:#?}"),
            },
            // ISSUE #000 handle-the-wro-work-order.
            WorkOrderType::Wro(wro_priority) => match wro_priority {
                Priority::Int(int) if (&0..=&8).contains(&int) => {
                    work_order_configurations.wgn_priority_map[&int.to_string()]
                        * work_order_configurations.order_type_weights["WGN"]
                }
                Priority::Char(char) if (&'A'..=&'D').contains(&char) => {
                    work_order_configurations.wpm_priority_map[char]
                        * work_order_configurations.order_type_weights["WPM"]
                }
                _ => bail!("Received a wrong input number work order priority: {wro_priority:#?}"),
            },
            WorkOrderType::Other => work_order_configurations.order_type_weights["Other"],
        };

        let status_weight = {
            let mut work_order_value = 0;
            if self.work_order_analytic.user_status_codes.awsc {
                work_order_value += work_order_configurations.status_weights["AWSC"];
            }

            if self.work_order_analytic.user_status_codes.sece {
                work_order_value += work_order_configurations.status_weights["SECE"];
            }

            if self.work_order_analytic.system_status_codes.pcnf
                && self.work_order_analytic.system_status_codes.nmat
                || self.work_order_analytic.user_status_codes.smat
            {
                work_order_value += work_order_configurations.status_weights["PCNF_NMAT_SMAT"];
            }

            work_order_value
        };

        let weight = (base_value + status_weight)
            * (self
                .work_order_load()
                .context("An error occured line creating the work_order_load")?
                .values()
                .map(|wor| wor.to_f64())
                .sum::<f64>() as u64);
        Ok(weight)
    }

    pub fn work_order_load(&self) -> Result<HashMap<Skill, Work>>
    {
        self.operations
            .0
            .values()
            .try_fold(HashMap::default(), |mut acc, ele_opr: &Operation| {
                ensure!(
                    ele_opr.operation_info.work_remaining >= Work::from(0.0),
                    "{:#?}",
                    ele_opr
                );
                *acc.entry(ele_opr.resource).or_insert(Work::from(0.0)) +=
                    ele_opr.operation_info.work_remaining.round();
                Ok(acc)
            })
    }

    /// This method determines that earliest allow start date and period for the
    /// work order. This is a maximum of the material status and the
    /// earliest start period of the operations. TODO : A stance will have
    /// to be taken on the VEN, SHUTDOWN, and SUBNETWORKS. We will get an
    /// error here! The problem is that after this the EASD will not be
    /// contained anymore.
    // TODO [ ]
    // Extract these parameters into a config file.
    // TODO [ ]
    // Move this code into the Builder
    pub fn find_excluded_periods(
        &self,
        periods: &[Period],
        material_to_period: &MaterialToPeriod,
    ) -> HashSet<Period>
    {
        periods
            .iter()
            .enumerate()
            .filter(|(i, per)| {
                *per < self.earliest_allowed_start_period(periods, material_to_period)
                    || (self.vendor() && *i <= 3)
                    || (self.work_order_info.revision.shutdown() && *i <= 3)
            })
            .map(|(_, per)| per.clone())
            .collect()
    }

    pub fn functional_location(&self) -> &FunctionalLocation
    {
        &self.work_order_info.functional_location
    }

    pub fn insert_operation(&mut self, operation: Operation)
    {
        self.operations.0.insert(operation.activity, operation);
    }

    // QUESTION
    // What should this function do? I think that the best approach is to
    // create something that will
    pub fn date_to_period<'a>(periods: &'a [Period], date_time: &NaiveDate) -> &'a Period
    {
        let period: Option<&Period> = periods.iter().find(|period| {
            period.start_datetime().date_naive() <= *date_time
                && period.finish_datetime().date_naive() >= *date_time
        });

        // This is created in a horrible way. I think that the best approach here
        // is to make.
        // You are effectively making a new instance period which is not. Should
        // the work orders age? That is the fundamental question? I do not believe
        // so. One thing is for sure, the old period should be in the
        // `SchedulingEnvironment::time_environment`.
        match period {
            Some(period) => period,
            None => periods.first().unwrap(),
        }
    }

    // FIX
    // This should be based on a different formulation. This whole thing should be
    // formulated differently
    pub fn earliest_allowed_start_period<'a>(
        &'a self,
        periods: &'a [Period],
        material_to_periods: &MaterialToPeriod,
    ) -> &'a Period
    {
        // This whole thing is bull shit.
        // TODO [ ]
        //

        // This is also not this straihtforward. There is an interplay between
        // this and the
        // assert!(
        //     self.earliest_allowed_start_period(&periods)
        //         .end_date()
        //         .date_naive()
        //         >= self.work_order_dates.earliest_allowed_start_date
        // );
        let period =
            Self::date_to_period(periods, &self.work_order_dates.earliest_allowed_start_date);

        match (&self.work_order_analytic.user_status_codes).into() {
            MaterialStatus::Nmat => (&periods[material_to_periods.nmat]).max(period),
            MaterialStatus::Smat => (&periods[material_to_periods.smat]).max(period),
            MaterialStatus::Cmat => (&periods[material_to_periods.cmat]).max(period),
            MaterialStatus::Pmat => (&periods[material_to_periods.pmat]).max(period),
            MaterialStatus::Wmat => (&periods[material_to_periods.wmat]).max(period),
            MaterialStatus::Unknown => panic!("WorkOrder does not have a material status"),
        }
    }

    pub fn latest_allowed_finish_period<'a>(&'a self, periods: &'a [Period]) -> &'a Period
    {
        Self::date_to_period(periods, &self.work_order_dates.latest_allowed_finish_date)
    }

    // fn random_latest_periods(&mut self, periods: &[Period]) {
    //     let mut rng = thread_rng();
    //     let random_period = periods.choose(&mut rng).unwrap();
    //     self.work_order_dates.latest_allowed_finish_period =
    // random_period.clone(); }
}
impl FromStr for WorkOrderNumber
{
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let number = s.parse::<u64>()?;
        Ok(Self(number))
    }
}

impl From<u64> for WorkOrderNumber
{
    fn from(value: u64) -> Self
    {
        WorkOrderNumber(value)
    }
}

impl Serialize for WorkOrderNumber
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkOrderNumber
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let work_order_number_string = String::deserialize(deserializer).unwrap();
        let work_order_number_primitive = work_order_number_string.parse::<u64>().unwrap();
        Ok(WorkOrderNumber(work_order_number_primitive))
    }
}

// TODO [ ]
// This is a horrible practice! You should refactor it.
impl WorkOrder
{
    pub fn work_order_test() -> Result<Self>
    {
        Ok(WorkOrder::builder(WorkOrderNumber(2100000001))
            .main_work_center(Skill::MtnMech)
            .operations_builder(10, Skill::Prodtech, |e| {
                e.operation_info(|oi| oi.number(1).work_remaining(10.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })?
            .operations_builder(20, Skill::MtnMech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(20.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })?
            .operations_builder(20, Skill::MtnMech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(30.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })?
            .operations_builder(40, Skill::Prodtech, |ob| {
                ob.operation_info(|oi| oi.number(1).work_remaining(40.0))
                    .operation_analytic(|e| e.preparation_time(0.0).duration(1.0))
                    .operation_dates(|b| {
                        b.earliest_start_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                        .earliest_finish_datetime(
                            DateTime::parse_from_rfc3339("2025-04-04T12:00:00Z")
                                .unwrap()
                                .to_utc(),
                        )
                    })
            })?
            .work_order_analytic_builder(|woab| {
                woab.system_status_codes(|sta| sta.rel(true))
                    .user_status_codes(|sta| sta.smat(true))
            })
            .work_order_info_builder(|woib| {
                // FIX [ ]
                // This is wrong and it should be fixed. You should make the
                // code work correctly no matter what.
                woib.priority(Priority::Int(1))
                    .work_order_type(WorkOrderType::Wdf(Priority::Int(1)))
            })
            .work_order_dates_builder(|wodb| {
                wodb.earliest_allowed_start_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
            })
            .build())
    }
}

pub struct WorkOrderView
{
    pub work_order_number: WorkOrderNumber,
    pub priority: String,
    pub revision: String,
    pub work_order_type: String,
    pub main_work_ctr: String,
    pub description_work_order: String,
    pub material_status: MaterialStatus,
    pub system_status: SystemStatusCodes,
    pub user_status: UserStatusCodes,
    pub operations: Vec<OperationView>,
    pub vendor: bool,
    pub basic_start_date: String,
    pub basic_finish_date: String,
    pub earliest_allowed_start_date: String,
    pub latest_allowed_finish_date: String,
    pub functional_location: String,
    pub subnetwork_of: String,
    pub system_condition: String,
    pub maintenance_plan: String,
    pub planner_group: String,
    pub maintenance_plant: String,
    pub pm_collective: String,
    pub room: String,
}

#[derive(Clone)]
pub struct OperationView
{
    pub activity: u64,
    pub resource: Skill,
    pub remaining_work: Work,
    pub actual_work: Work,
    pub number_of_people: u64,
    pub unloading_point: String,
    pub duration: Work,
    pub earliest_start_datetime: DateTime<Utc>,
    pub earliest_finish_datetime: DateTime<Utc>,
    pub operation_short_text: String,
    pub description_operation: String,
    // operation_system_status: String,
    // operation_user_status: String,
}

/// Read models should be implemented here.
impl WorkOrder
{
    pub fn view(&self) -> WorkOrderView
    {
        let mut operation_views: Vec<OperationView> = vec![];

        for operation in &self.operations.0 {
            let operation_view = operation.1.view();
            operation_views.push(operation_view);
        }

        WorkOrderView {
            work_order_number: self.work_order_number,
            priority: self.work_order_info.priority.to_string(),
            revision: self.work_order_info.revision.to_string(),
            work_order_type: self.work_order_info.work_order_type.to_string(),
            main_work_ctr: self.main_work_center.to_string(),
            description_work_order: self
                .work_order_info
                .work_order_text
                .order_description
                .clone(),

            material_status: MaterialStatus::from(&self.work_order_analytic.user_status_codes),
            system_status: self.work_order_analytic.system_status_codes.clone(),
            user_status: self.work_order_analytic.user_status_codes.clone(),
            basic_start_date: self.work_order_dates.basic_start_date.to_string(),
            basic_finish_date: self.work_order_dates.basic_finish_date.to_string(),
            earliest_allowed_start_date: self
                .work_order_dates
                .earliest_allowed_start_date
                .to_string(),
            latest_allowed_finish_date: self
                .work_order_dates
                .latest_allowed_finish_date
                .to_string(),
            operations: operation_views,
            // operation_system_status: self.status_codes().clone(),
            // operation_user_status: self.status_codes().clone(),
            functional_location: self.functional_location().to_string(),
            subnetwork_of: self
                .work_order_info
                .work_order_info_detail
                .subnetwork
                .clone(),
            system_condition: self.work_order_info.system_condition.to_string(),
            maintenance_plan: self
                .work_order_info
                .work_order_info_detail
                .maintenance_plan
                .clone(),
            planner_group: self
                .work_order_info
                .work_order_info_detail
                .planner_group
                .clone(),
            maintenance_plant: self
                .work_order_info
                .work_order_info_detail
                .maintenance_plant
                .clone(),
            pm_collective: self
                .work_order_info
                .work_order_info_detail
                .pm_collective
                .clone(),
            room: self.work_order_info.work_order_info_detail.room.clone(),
            vendor: self.vendor(),
        }
    }
}
// The issue with what you are doing is that we can keep implementing stuff like
// this until the day we die. You have to simply go for the money here. I think
// that is the best approach.
//
// You are not good enough to code. You are good enough to do this, Brian
// believes in you. You simply have to keep working.

#[cfg(test)]
mod tests
{
    use std::str::FromStr;

    use super::WorkOrder;
    // #[test]
    // fn test_initialize_work_load()
    // {
    //     let work_order = WorkOrder::work_order_test();

    //     assert_eq!(
    //         *work_order
    //             .work_order_load()
    //             .unwrap()
    //             .get(&Resources::from_str("PRODTECH").unwrap())
    //             .unwrap(),
    //         Work::from(50.0)
    //     );
    //     assert_eq!(
    //         *work_order
    //             .work_order_load()
    //             .unwrap()
    //             .get(&Resources::from_str("MTN-MECH").unwrap())
    //             .unwrap(),
    //         Work::from(50.0)
    //     );
    // }
    use super::*;

    #[test]
    fn test_date_to_period()
    {
        let periods: Vec<Period> = vec![
            Period::from_str("2024-W47-48").unwrap(),
            Period::from_str("2024-W49-50").unwrap(),
            Period::from_str("2024-W51-52").unwrap(),
            Period::from_str("2025-W1-2").unwrap(),
        ];

        let period_1 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2024, 12, 5).unwrap(),
        );
        let period_2 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2024, 12, 27).unwrap(),
        );
        let period_3 = WorkOrder::date_to_period(
            periods.as_slice(),
            &NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
        );

        assert_eq!(period_1, periods.get(1).unwrap());
        assert_eq!(period_2, periods.get(2).unwrap());
        assert_eq!(period_3, periods.get(3).unwrap());
    }
}
