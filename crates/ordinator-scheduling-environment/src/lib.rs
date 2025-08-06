#![feature(iter_map_windows)]
pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::{self};
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::EnumIter;
use time_environment::TimeEnvironmentBuilder;
use time_environment::day::Day;
use time_environment::period::Period;
use work_order::WorkOrderNumber;
use work_order::WorkOrders;
use work_order::WorkOrdersBuilder;
use work_order::operation::ActivityNumber;
use worker_environment::resources::Id;

use self::time_environment::TimeEnvironment;
use self::worker_environment::ActorEnvironment;

#[derive(Deserialize, Serialize, Debug)]
pub struct SchedulingEnvironment
{
    pub work_orders: WorkOrders,
    pub worker_environment: ActorEnvironment,
    pub time_environment: TimeEnvironment,

    pub assignments: SavedAssignment,
    // material
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SavedAssignment(Vec<(WorkOrderNumber, Option<ActivityNumber>, Assignment)>);

// FORGET:
// * Exclude
// * Correct data structure
// * Keep it simple
// This is everything I believe.
#[derive(Deserialize, Serialize, Debug)]
pub struct Assignment
{
    period: Option<Period>,
    day: Option<Day>,
    technician: HashSet<(Id, DateTime<Utc>)>,
}

impl Assignment
{
    fn new(
        period: Option<Period>,
        day: Option<Day>,
        technician: HashSet<(Id, DateTime<Utc>)>,
    ) -> Self
    {
        Self {
            period,
            day,
            technician,
        }
    }
}

// `new` and modification is very different here. You should clearly understand
// the difference here.
//

// You have to make this work. That is the only thing that matters.

// ESSAY [ ] How should the state interact?
// I think that the best approach here is to make the system work on the
// correct... Your issue here is that you need to respect a lot of things
// in the work order, and the work order already has information about the
// person that should do the job. This is a major problem. I think that the
// best model here is to pull out all the `time` and `worker` state of the
// `WorkOrders` and create something that will make it easy to reference the
// correct thing. This means that some of the logic in the `WorkOrder` will
// have to be pulled out of the struct. The other approach here is to make
// the system work as a state machine. So to round up here. The state should
// be completely separate, and the `fixed_by` here should simply reference the
// correct elements. I do not see what other approach to choose.
//
// ESSAY: [ ] How would a state machine function here?
// WorkOrder should be in a range of different states here. And depending on
// what is needs we should treat the code differently.
// We should create this function so that it will
// All endpoints should change both the StateMachine and the WorkOrder (if
// required)
//
// Do not think about DDD at the moment. Simply make the data structure
// to support two different kinds of

pub struct SchedulingEnvironmentBuilder
{
    work_orders: Option<WorkOrders>,
    worker_environment: Option<ActorEnvironment>,
    time_environment: Option<TimeEnvironment>,
}

impl SchedulingEnvironment
{
    pub fn builder() -> SchedulingEnvironmentBuilder
    {
        SchedulingEnvironmentBuilder {
            work_orders: None,
            worker_environment: None,
            time_environment: None,
        }
    }
}

pub trait IntoSchedulingEnvironment
{
    type S: SystemConfigurationTrait;

    fn into_scheduling_environment(
        self,
        current_time: DateTime<Utc>,
        system_configuration: &Self::S,
    ) -> Result<Arc<Mutex<SchedulingEnvironment>>>;
}

pub trait SystemConfigurationTrait {}

pub trait DatabaseConfigurationTrait {}

impl SchedulingEnvironmentBuilder
{
    // QUESTION
    // Do you believe that this is the most appropriate way of structuring the code
    // here? Yes I think that this is the best way of doing it.
    pub fn build(mut self) -> Arc<Mutex<SchedulingEnvironment>>
    {
        // The `WorkOrder` have to help in deriving the `SavedAssignment`.
        //
        let work_orders = self
            .work_orders
            .expect("You should build the WorkOrders with the correct parameters injected.");

        let mut assignments = Vec::new();

        let time_environment = self
            .time_environment
            .expect("Time environment should be present");

        let worker_environment = self
            .worker_environment
            .take()
            .expect("ActorEnvironment should always be available.");

        let material_to_period = &worker_environment
            .actor_specification
            .iter()
            .nth(0)
            .unwrap()
            .1
            .material_to_period;

        for (work_order_number, work_order) in &work_orders.inner {
            // This methods is created in exactly the same way here.
            // The fundamental issue is that you do not understand the business logic here.

            //
            let forced_work_order = work_order
                .forced_work_order(
                    &time_environment.periods,
                    &time_environment.days,
                    material_to_period,
                )
                .unwrap();

            // We want to make the `ForcedWorkOrder` and turn it into the other aggregate!
            // Yes that is the approach forward. o
            //
            //
            // You should not be smart, scalable.

            let assignment = Assignment::new(None, None, HashSet::default());
            assignments.push((*work_order_number, None, assignment));
        }
        let saved_assignments = SavedAssignment(assignments);
        Arc::new(Mutex::new(SchedulingEnvironment {
            work_orders,
            worker_environment,
            time_environment,
            assignments: saved_assignments,
        }))
    }

    pub fn time_environment(mut self, time_environment: TimeEnvironment) -> Self
    {
        self.time_environment = Some(time_environment);
        self
    }

    pub fn time_environment_builder<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut TimeEnvironmentBuilder) -> &mut TimeEnvironmentBuilder,
    {
        let mut time_environment_builder = TimeEnvironmentBuilder::default();

        f(&mut time_environment_builder);

        self.time_environment = Some(time_environment_builder.build());
        self
    }

    pub fn worker_environment(mut self, worker_environment: ActorEnvironment) -> Self
    {
        self.worker_environment = Some(worker_environment);
        self
    }

    pub fn work_orders(mut self, work_orders: WorkOrders) -> Self
    {
        self.work_orders = Some(work_orders);
        self
    }

    pub fn work_orders_builder<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WorkOrdersBuilder) -> WorkOrdersBuilder,
    {
        let work_orders_builder = WorkOrders::builder();

        let work_orders_builder = f(work_orders_builder);

        self.work_orders = Some(work_orders_builder.build());
        self
    }
}

// Do we want this? Yes I think that is a really good idea.
impl fmt::Display for SchedulingEnvironment
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let workers = self
            .worker_environment
            .actor_specification
            .iter()
            .map(|e| e.1.operational.len())
            .sum::<usize>();
        write!(
            f,
            "The Scheduling Environment is currently comprised of
        \n  number of work orders: {}
        \n  number of worker entries: {}
        \n  number of strategic periods: {},
        \n  number of tactical days: {}",
            self.work_orders.inner.len(),
            workers,
            self.time_environment.periods.len(),
            self.time_environment.days.len(),
        )?;
        Ok(())
    }
}

// TODO [ ]
// Move to configuration files
#[derive(PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize, Debug, Clone, EnumIter)]
pub enum Asset
{
    DF,
    DM,
    DE,
    GO,
    HB,
    HC,
    HD,
    HW,
    KR,
    RO,
    RF,
    SK,
    SV,
    TE,
    TS,
    VA,
    VB,
    Unknown,
    Test,
}

impl Display for Asset
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Asset::DF => write!(f, "DF"),
            Asset::DM => write!(f, "DM"),
            Asset::DE => write!(f, "DE"),
            Asset::GO => write!(f, "GO"),
            Asset::HB => write!(f, "HB"),
            Asset::HC => write!(f, "HC"),
            Asset::HD => write!(f, "HD"),
            Asset::HW => write!(f, "HW"),
            Asset::KR => write!(f, "KR"),
            Asset::RO => write!(f, "RO"),
            Asset::RF => write!(f, "RF"),
            Asset::SK => write!(f, "SK"),
            Asset::SV => write!(f, "SV"),
            Asset::TE => write!(f, "TE"),
            Asset::TS => write!(f, "TS"),
            Asset::VA => write!(f, "VA"),
            Asset::VB => write!(f, "VB"),
            Asset::Test => write!(f, "TEST"),
            Asset::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Asset
{
    pub fn new_from_string(asset_string: &str) -> Option<Asset>
    {
        match asset_string {
            "DF" => Some(Asset::DF),
            "DM" => Some(Asset::DM),
            "DE" => Some(Asset::DE),
            "GO" => Some(Asset::GO),
            "HB" => Some(Asset::HB),
            "HC" => Some(Asset::HC),
            "HD" => Some(Asset::HD),
            "HW" => Some(Asset::HW),
            "KR" => Some(Asset::KR),
            "RO" => Some(Asset::RO),
            "RF" => Some(Asset::RF),
            "SK" => Some(Asset::SK),
            "SV" => Some(Asset::SV),
            "TE" => Some(Asset::TE),
            "TS" => Some(Asset::TS),
            "VA" => Some(Asset::VA),
            "VB" => Some(Asset::VB),
            "TEST" => Some(Asset::Test),
            "test" => Some(Asset::Test),
            "Test" => Some(Asset::Test),
            _ => None,
        }
    }
}
