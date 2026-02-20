#![feature(iter_map_windows)]
pub mod assignments;
pub mod materials;
pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::{self};
use std::fs::File;
use std::io::Read;
use std::option::Option;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use assignments::Assignment;
use assignments::SavedAssignment;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use materials::MaterialRepo;
use materials::MaterialToPeriod;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::EnumIter;
use time_environment::TimeEnvironmentBuilder;
use time_environment::create_time_environment;
use time_environment::period::Period;
use uuid::Uuid;
use valuable::Valuable;
use work_order::WorkOrderPolicies;
use work_order::WorkOrders;
use work_order::WorkOrdersBuilder;
use worker_environment::ActorSpecification;
use worker_environment::ActorSpecifications;
use worker_environment::TimeInput;

use self::time_environment::TimeEnvironment;
use self::worker_environment::ActorEnvironment;

// ESSAY: #20250814
// All of these should be `dyn`. If you need Serialize and Deserialize, implement on concrete types.
/// Main entry point to the domain models and aggregate roots
#[derive(Debug)]
pub struct SchedulingEnvironment
{
    pub work_orders: WorkOrders,
    pub worker_environment: ActorEnvironment<dyn ActorSpecification>,
    pub time_environment: TimeEnvironment,

    pub work_order_policies: WorkOrderPolicies,
    pub material_repo: MaterialRepo,
    pub assignments: SavedAssignment,
}

pub enum TimeType
{
    Period(Period),
    Day(NaiveDate),
    SpecificTime(DateTime<Utc>),
}

// TODO: Consider state machine pattern for WorkOrder state transitions and actor assignment
#[allow(dead_code)]
pub struct SchedulingEnvironmentBuilder
{
    work_orders: Option<WorkOrders>,
    worker_environment: Option<ActorEnvironment<dyn ActorSpecification>>,
    time_environment: Option<TimeEnvironment>,
    work_order_policies: Option<WorkOrderPolicies>,

    material_repo: Option<MaterialRepo>,
    assignments: Option<SavedAssignment>,
}

impl SchedulingEnvironment
{
    pub fn builder() -> SchedulingEnvironmentBuilder
    {
        SchedulingEnvironmentBuilder {
            work_orders: None,
            worker_environment: None,
            time_environment: None,
            work_order_policies: None,
            material_repo: None,
            assignments: None,
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

// ISSUE #000 - Convert builder to typestate pattern
impl SchedulingEnvironmentBuilder
{
    /// Build the SchedulingEnvironment with all configured components
    pub fn build(mut self) -> Result<Arc<Mutex<SchedulingEnvironment>>>
    {
        let work_orders = self
            .work_orders
            .context("You should build the WorkOrders with the correct parameters injected.")?;

        let mut assignments = HashMap::new();

        let time_environment = self
            .time_environment
            .context("Time environment should be present")?;

        let worker_environment = self
            .worker_environment
            .take()
            .context("ActorEnvironment should always be available.")?;

        for work_order_number in work_orders.inner.keys() {
            // ISSUE #002 - Create AssignmentRepo and derive ForcedWorkOrder
            let assignment = assignments::AnyAssignment::Base(Assignment::new(
                *work_order_number,
                None,
                None,
                None,
                HashSet::default(),
            ));
            assignments.insert(Uuid::new_v4(), assignment);
        }
        let saved_assignments = SavedAssignment::new(assignments);
        // ISSUE #TODO - Create typestate builder for SchedulingEnvironment

        let work_order_policies = self
            .work_order_policies
            .context("WorkOrderPolicies not added to SchedulingEnvironmentBuilder")?;
        let material_repo = self
            .material_repo
            .context("MaterialRepo not added to the SchedulingEnvironmentBuilder")?;
        Ok(Arc::new(Mutex::new(SchedulingEnvironment {
            work_orders,
            worker_environment,
            time_environment,
            assignments: saved_assignments,
            work_order_policies,
            material_repo,
        })))
    }

    pub fn time_environment(mut self, time_environment: TimeEnvironment) -> Self
    {
        self.time_environment = Some(time_environment);
        self
    }

    pub fn work_order_policies(mut self, work_order_policies: WorkOrderPolicies) -> Self
    {
        self.work_order_policies = Some(work_order_policies);
        self
    }

    pub fn material_repo(mut self, material_repo: MaterialRepo) -> Self
    {
        self.material_repo = Some(material_repo);
        self
    }

    pub fn time_environment_from_toml(
        mut self,
        path_to_time_environment: PathBuf,
        current_time: DateTime<Utc>,
    ) -> Result<Self>
    {
        let time_input_string = std::fs::read_to_string(path_to_time_environment)
            .with_context(|| format!("Could not load TimeEnvironment Config. {}", line!()))?;

        let time_input: TimeInput = toml::from_str(&time_input_string).with_context(|| {
            format!("Could not deserialize the TimeInput config. Input:\n{time_input_string}")
        })?;

        let time_environment = create_time_environment(current_time, &time_input);

        self.time_environment = Some(time_environment);
        Ok(self)
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

    pub fn work_orders(mut self, work_orders: WorkOrders) -> Self
    {
        self.work_orders = Some(work_orders);
        self
    }

    pub fn work_orders_from_json(mut self, path_to_work_orders: PathBuf) -> Result<Self>
    {
        let mut file = File::open(path_to_work_orders)?;
        let mut data = String::new();

        file.read_to_string(&mut data)?;

        let work_orders = serde_json::from_str::<WorkOrders>(&data).context("Could not build the WorkOrders from the JSON file\n1. Did you modify the schema (WorkOrders)?\n2. Is the data corrupted?")?;
        self.work_orders = Some(work_orders);
        Ok(self)
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

    pub fn worker_environment(
        mut self,
        worker_environment: ActorEnvironment<dyn ActorSpecification>,
    ) -> Self
    {
        self.worker_environment = Some(worker_environment);
        self
    }

    pub fn worker_environment_from_toml(
        mut self,
        path_to_workers: PathBuf,
        asset: Asset,
    ) -> anyhow::Result<Self>
    {
        let actor_environment = ActorEnvironment::<dyn ActorSpecification>::builder()
            .add_actor_specification(
                asset.clone(),
                Box::new(ActorSpecifications::actor_specification_from_toml(
                    path_to_workers,
                )?),
            )
            .build();

        self.worker_environment = Some(actor_environment);
        Ok(self)
    }

    pub fn add_actor_specification<F>(mut self, asset: Asset, f: F) -> Self
    where
        F: FnOnce(
            worker_environment::ActorSpecificationBuilder,
        ) -> worker_environment::ActorSpecificationBuilder,
    {
        let actor_spec_builder = ActorSpecifications::builder();
        let actor_spec_builder = f(actor_spec_builder);

        let actor_spec = actor_spec_builder
            .build()
            .expect("Failed to build ActorSpecifications");

        let actor_environment = self
            .worker_environment
            .take()
            .unwrap_or_else(|| ActorEnvironment::<dyn ActorSpecification>::builder().build())
            .actor_specification;

        let mut updated_environment = ActorEnvironment::<dyn ActorSpecification>::builder();

        for (existing_asset, existing_spec) in actor_environment {
            updated_environment =
                updated_environment.add_actor_specification(existing_asset, existing_spec);
        }

        updated_environment =
            updated_environment.add_actor_specification(asset, Box::new(actor_spec));

        self.worker_environment = Some(updated_environment.build());
        self
    }

    pub fn work_order_policies_from_toml(
        mut self,
        path_to_work_order_policies: PathBuf,
    ) -> Result<Self>
    {
        let contents = std::fs::read_to_string(path_to_work_order_policies).unwrap();
        let work_order_policies: WorkOrderPolicies =
            toml::from_str(&contents).expect("Could not read WorkOrderPolicies");

        self.work_order_policies = Some(work_order_policies);
        Ok(self)
    }

    pub fn material_repo_from_toml(mut self, path_to_material_to_period: PathBuf) -> Result<Self>
    {
        let material_to_period_string =
            std::fs::read_to_string(path_to_material_to_period).unwrap();

        let material_to_period: MaterialToPeriod =
            toml::from_str(&material_to_period_string).unwrap();

        let material_repo = MaterialRepo::new(material_to_period);

        self.material_repo = Some(material_repo);
        Ok(self)
    }
}

impl fmt::Display for SchedulingEnvironment
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let workers = self
            .worker_environment
            .actor_specification
            .iter()
            .map(|e| e.1.operational().len())
            .sum::<usize>();
        write!(
            f,
            "The Scheduling Environment is currently comprised of
        \n  number of work orders: {}
        \n  number of worker entries: {}
        \n  number of weekly periods: {},
        \n  number of project days: {}",
            self.work_orders.inner.len(),
            workers,
            self.time_environment.periods.len(),
            self.time_environment.days.len(),
        )?;
        Ok(())
    }
}

// TODO - Move Asset variants to configuration files
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
    /// Matches an asset value as string to Asset variant.
    pub fn new_from_string(asset_string: &str) -> Option<Asset>
    {
        // NOTE: Matching requires uppercase conversion
        match asset_string.to_uppercase().as_str() {
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
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Valuable, PartialOrd, Ord)]
pub struct Percent
{
    percent: u64,
    numerator: u64,
    denominator: u64,
}

impl Percent
{
    pub fn new(numerator: u64, denominator: u64) -> anyhow::Result<Self>
    {
        match (100 * numerator).checked_div(denominator) {
            Some(percent) => Ok(Self {
                percent,
                numerator,
                denominator,
            }),
            None => Ok(Self {
                percent: 0,
                numerator: 0,
                denominator: 0,
            }),
        }
    }

    pub fn percent(&self) -> u64
    {
        self.percent
    }
}
