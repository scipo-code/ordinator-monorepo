pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::panic::Location;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use availability::Availability;
use chrono::DateTime;
use chrono::Utc;
use crew::OperationalConfiguration;
use crew::OperationalConfigurationBuilder;
use resources::ActorCompositeId;
use resources::Skill;
use serde::Deserialize;
use serde::Serialize;

use crate::Asset;
use crate::time_environment::TimeInterval;
use crate::work_order::WorkOrderPolicies;

pub type OperationalId = String;

// TODO: Improve system behavior prediction model
#[derive(Deserialize, Serialize, Debug)]
pub struct ActorEnvironment<A>
where
    A: ActorSpecification + Debug + ?Sized,
{
    pub actor_specification: HashMap<Asset, Box<A>>,
}

pub struct ActorEnvironmentBuilder<A>
where
    A: ActorSpecification + Debug + ?Sized,
{
    pub actor_environment: HashMap<Asset, Box<A>>,
}

impl<A> ActorEnvironment<A>
where
    A: ActorSpecification + Debug + ?Sized,
{
    // TODO: Refactor builder pattern
    pub fn builder() -> ActorEnvironmentBuilder<A>
    {
        ActorEnvironmentBuilder {
            actor_environment: HashMap::default(),
        }
    }
}

pub enum EmptyFull
{
    Empty,
    Full,
}

impl<A> ActorEnvironmentBuilder<A>
where
    A: ActorSpecification + ?Sized + Debug,
{
    pub fn build(self) -> ActorEnvironment<A>
    {
        ActorEnvironment {
            actor_specification: self.actor_environment,
        }
    }

    pub fn add_actor_specification(mut self, asset: Asset, actor_specification: Box<A>) -> Self
    {
        self.actor_environment.insert(asset, actor_specification);
        self
    }
}

pub trait ActorSpecification: Send + Sync + Debug
{
    fn weekly_options(&self) -> &WeeklyOptions;

    fn weekly(&self) -> &InputWeekly;

    fn project(&self) -> &InputProject;

    fn daily(&self) -> &Vec<InputDaily>;

    fn operational(&self) -> &HashMap<IdString, InputOperational>;

    fn technician_availability(
        &self,
    ) -> BTreeMap<IdString, (BTreeSet<Availability>, HashSet<Skill>)>;

    fn add_operational(
        &mut self,
        id: &IdString,
        assets: Vec<Asset>,
        resources: Vec<Skill>,
        start_date: DateTime<Utc>,
        finish_date: DateTime<Utc>,
    ) -> anyhow::Result<ActorCompositeId>;
}

// TODO: Create trait implementation for IdString
pub type IdString = String;
#[derive(Serialize, Deserialize, Debug)]
pub struct ActorSpecifications
{
    pub weekly: InputWeekly,
    pub project: InputProject,
    pub dailys: Vec<InputDaily>,
    pub operational: HashMap<IdString, InputOperational>,
    // TODO: Consider using relational database structure instead of HashMap
}

impl ActorSpecification for ActorSpecifications
{
    fn operational(&self) -> &HashMap<IdString, InputOperational>
    {
        &self.operational
    }

    fn weekly_options(&self) -> &WeeklyOptions
    {
        &self.weekly.weekly_options
    }

    fn daily(&self) -> &Vec<InputDaily>
    {
        &self.dailys
    }

    fn project(&self) -> &InputProject
    {
        &self.project
    }

    fn weekly(&self) -> &InputWeekly
    {
        &self.weekly
    }

    fn technician_availability(
        &self,
    ) -> BTreeMap<IdString, (BTreeSet<Availability>, HashSet<Skill>)>
    {
        self.operational
            .iter()
            .map(|e| {
                (
                    e.0.clone(),
                    (
                        e.1.operational_configuration.availability.clone(),
                        e.1.operational_configuration.resources.clone(),
                    ),
                )
            })
            .collect()
    }

    fn add_operational(
        &mut self,
        id: &IdString,
        assets: Vec<Asset>,
        resources: Vec<Skill>,
        start_date: DateTime<Utc>,
        finish_date: DateTime<Utc>,
    ) -> anyhow::Result<ActorCompositeId>
    {
        let availability = Availability::new(start_date, finish_date, assets)?;

        self.operational
            .entry(id.clone())
            .and_modify(|e| {
                e.operational_configuration
                    .availability
                    .insert(availability.clone());
            })
            .or_insert(InputOperational::new(
                id.clone(),
                resources.clone(),
                6.0,
                availability.clone(),
            ));

        Ok(ActorCompositeId::new(id, resources, availability))
    }
}

impl ActorSpecifications
{
    pub fn builder() -> ActorSpecificationBuilder
    {
        ActorSpecificationBuilder::new()
    }

    pub fn actor_specification_from_toml(path_to_data: PathBuf) -> Result<Self>
    {
        println!("{}", std::env::current_dir()?.display());
        let contents = std::fs::read_to_string(&path_to_data).with_context(|| {
            format!(
                "Could not read string for ActorSpecification\nPath: {}",
                path_to_data.display()
            )
        })?;

        let actor_specifications: Self  =
            toml::from_str(&contents).with_context(|| {
                format!(
                    "Could not deserialize into ActorSpecification. Location: {}\nContent String\n{contents}",
                    Location::caller()
                )
            })?;

        Ok(actor_specifications)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeInput
{
    pub number_of_periods: u64,
    pub number_of_days: u64,
}

pub struct ActorSpecificationBuilder
{
    weekly: Option<InputWeekly>,
    project: Option<InputProject>,
    dailys: Option<Vec<InputDaily>>,
    operational: Option<HashMap<IdString, InputOperational>>,
}

impl ActorSpecificationBuilder
{
    pub fn new() -> Self
    {
        Self {
            weekly: None,
            project: None,
            dailys: None,
            operational: None,
        }
    }

    pub fn weekly<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputWeeklyBuilder) -> InputWeeklyBuilder,
    {
        let weekly_builder = InputWeekly::builder();
        let weekly_builder = f(weekly_builder);
        self.weekly = Some(weekly_builder.build());
        self
    }

    pub fn project<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputProjectBuilder) -> InputProjectBuilder,
    {
        let project_builder = InputProject::builder();
        let project_builder = f(project_builder);
        self.project = Some(project_builder.build());
        self
    }

    pub fn dailys<F>(mut self, f: F) -> Self
    where
        F: FnOnce(DailysBuilder) -> DailysBuilder,
    {
        let dailys_builder = DailysBuilder::new();
        let dailys_builder = f(dailys_builder);
        self.dailys = Some(dailys_builder.build());
        self
    }

    pub fn operational<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OperationalBuilder) -> OperationalBuilder,
    {
        let operational_builder = OperationalBuilder::new();
        let operational_builder = f(operational_builder);
        self.operational = Some(operational_builder.build());
        self
    }

    pub fn build(self) -> Result<ActorSpecifications>
    {
        Ok(ActorSpecifications {
            weekly: self
                .weekly
                .ok_or_else(|| anyhow::anyhow!("Weekly configuration is required"))?,
            project: self
                .project
                .ok_or_else(|| anyhow::anyhow!("Project configuration is required"))?,
            dailys: self.dailys.unwrap_or_default(),
            operational: self.operational.unwrap_or_default(),
        })
    }
}

// TODO: Move work_order_parameters.json configuration here
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputWeekly
{
    pub id: IdString,
    pub number_of_weekly_periods: usize,
    pub weekly_options: WeeklyOptions,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputProject
{
    pub id: IdString,
    pub number_of_project_days: usize,
    pub project_options: ProjectOptions,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputDaily
{
    pub id: IdString,
    pub number_of_daily_periods: u64,
    pub daily_options: DailyOptions,
}

// TODO: Load IDs directly from configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputOperational
{
    pub id: IdString,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
    pub operational_options: OperationalOptions,
}

impl InputOperational
{
    pub fn new(
        id: IdString,
        resources: Vec<Skill>,
        hours_per_day: f64,

        availability: Availability,
    ) -> Self
    {
        let resources = resources.iter().cloned().collect::<HashSet<_>>();
        let availabilities = BTreeSet::from([availability]);
        let operational_configuration = OperationalConfiguration::new(
            availabilities,
            TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap(),
            TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
            TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap(),
            resources,
        );

        let operational_options = OperationalOptions {
            number_of_removed_activities: 15,
        };

        Self {
            id: id.clone(),
            hours_per_day,
            operational_configuration,
            operational_options,
        }
    }
}
/// Loads weekly configurations for use by agents
///
/// TODO: Resolve duplication between WeeklyOptions and database representation
/// TODO: Address StdRng configuration and custom deserialization
/// TODO: Reduce coupling between Actor and Orchestrator
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct WeeklyOptions
{
    pub number_of_removed_work_orders: usize,
    pub urgency_weight: usize,
    pub resource_penalty_weight: usize,
    pub clustering_weight: usize,
    pub work_order_policies: WorkOrderPolicies,
    // TODO: Move weight parameters to SchedulingEnvironment
}

// TODO: Move RNG configuration outside of ordinator-scheduling-environment
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct ProjectOptions
{
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct DailyOptions
{
    pub number_of_unassigned_work_orders: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct OperationalOptions
{
    pub number_of_removed_activities: usize,
}

pub struct InputWeeklyBuilder
{
    id: Option<IdString>,
    number_of_weekly_periods: Option<usize>,
    weekly_options: Option<WeeklyOptions>,
}

impl InputWeeklyBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_weekly_periods: None,
            weekly_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_weekly_periods(mut self, periods: usize) -> Self
    {
        self.number_of_weekly_periods = Some(periods);
        self
    }

    pub fn weekly_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WeeklyOptionsBuilder) -> WeeklyOptionsBuilder,
    {
        let options_builder = WeeklyOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.weekly_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputWeekly
    {
        InputWeekly {
            id: self.id.expect("id is required"),
            number_of_weekly_periods: self
                .number_of_weekly_periods
                .expect("number_of_weekly_periods is required"),
            weekly_options: self.weekly_options.expect("weekly_options is required"),
        }
    }
}

impl InputWeekly
{
    pub fn builder() -> InputWeeklyBuilder
    {
        InputWeeklyBuilder::new()
    }
}

pub struct InputProjectBuilder
{
    id: Option<IdString>,
    number_of_project_days: Option<usize>,
    project_options: Option<ProjectOptions>,
}

impl InputProjectBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_project_days: None,
            project_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_project_days(mut self, days: usize) -> Self
    {
        self.number_of_project_days = Some(days);
        self
    }

    pub fn project_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ProjectOptionsBuilder) -> ProjectOptionsBuilder,
    {
        let options_builder = ProjectOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.project_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputProject
    {
        InputProject {
            id: self.id.expect("id is required"),
            number_of_project_days: self
                .number_of_project_days
                .expect("number_of_project_days is required"),
            project_options: self.project_options.expect("project_options is required"),
        }
    }
}

impl InputProject
{
    pub fn builder() -> InputProjectBuilder
    {
        InputProjectBuilder::new()
    }
}

pub struct DailysBuilder
{
    dailys: Vec<InputDaily>,
}

impl DailysBuilder
{
    pub fn new() -> Self
    {
        Self { dailys: Vec::new() }
    }

    pub fn daily<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputDailyBuilder) -> InputDailyBuilder,
    {
        let daily_builder = InputDailyBuilder::new();
        let daily_builder = f(daily_builder);
        self.dailys.push(daily_builder.build());
        self
    }

    pub fn build(self) -> Vec<InputDaily>
    {
        self.dailys
    }
}

pub struct InputDailyBuilder
{
    id: Option<IdString>,
    number_of_daily_periods: Option<u64>,
    daily_options: Option<DailyOptions>,
}

impl Default for InputDailyBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl InputDailyBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_daily_periods: None,
            daily_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_daily_periods(mut self, periods: u64) -> Self
    {
        self.number_of_daily_periods = Some(periods);
        self
    }

    pub fn daily_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(DailyOptionsBuilder) -> DailyOptionsBuilder,
    {
        let options_builder = DailyOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.daily_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputDaily
    {
        InputDaily {
            id: self.id.expect("id is required"),
            number_of_daily_periods: self
                .number_of_daily_periods
                .expect("number_of_daily_periods is required"),
            daily_options: self.daily_options.expect("daily_options is required"),
        }
    }
}

pub struct OperationalBuilder
{
    operational: HashMap<IdString, InputOperational>,
}

impl OperationalBuilder
{
    pub fn new() -> Self
    {
        Self {
            operational: HashMap::new(),
        }
    }

    pub fn operational<F>(mut self, id: &str, f: F) -> Self
    where
        F: FnOnce(InputOperationalBuilder) -> InputOperationalBuilder,
    {
        let operational_builder = InputOperationalBuilder::new(id.to_string());
        let operational_builder = f(operational_builder);
        self.operational
            .insert(id.to_string(), operational_builder.build());
        self
    }

    pub fn build(self) -> HashMap<IdString, InputOperational>
    {
        self.operational
    }
}

pub struct InputOperationalBuilder
{
    id: IdString,
    hours_per_day: Option<f64>,
    operational_configuration: Option<OperationalConfiguration>,
    operational_options: Option<OperationalOptions>,
}

impl InputOperationalBuilder
{
    pub fn new(id: String) -> Self
    {
        Self {
            id,
            hours_per_day: None,
            operational_configuration: None,
            operational_options: None,
        }
    }

    pub fn hours_per_day(mut self, hours: f64) -> Self
    {
        self.hours_per_day = Some(hours);
        self
    }

    pub fn operational_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OperationalOptionsBuilder) -> OperationalOptionsBuilder,
    {
        let options_builder = OperationalOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.operational_options = Some(options_builder.build());
        self
    }

    pub fn operational_configuration<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OperationalConfigurationBuilder) -> OperationalConfigurationBuilder,
    {
        let config_builder = OperationalConfigurationBuilder::new();
        let config_builder = f(config_builder);
        self.operational_configuration = Some(config_builder.build());
        self
    }

    pub fn build(self) -> InputOperational
    {
        InputOperational {
            id: self.id,
            hours_per_day: self.hours_per_day.expect("hours_per_day is required"),
            operational_configuration: self
                .operational_configuration
                .expect("operational_configuration is required"),
            operational_options: self
                .operational_options
                .expect("operational_options is required"),
        }
    }
}

pub struct WeeklyOptionsBuilder
{
    number_of_removed_work_orders: Option<usize>,
    urgency_weight: Option<usize>,
    resource_penalty_weight: Option<usize>,
    clustering_weight: Option<usize>,
    pub work_order_policies: Option<WorkOrderPolicies>,
}

impl WeeklyOptionsBuilder
{
    pub fn new() -> Self
    {
        Self {
            number_of_removed_work_orders: None,
            urgency_weight: None,
            resource_penalty_weight: None,
            clustering_weight: None,
            work_order_policies: None,
        }
    }

    pub fn number_of_removed_work_orders(mut self, count: usize) -> Self
    {
        self.number_of_removed_work_orders = Some(count);
        self
    }

    pub fn urgency_weight(mut self, weight: usize) -> Self
    {
        self.urgency_weight = Some(weight);
        self
    }

    pub fn resource_penalty_weight(mut self, weight: usize) -> Self
    {
        self.resource_penalty_weight = Some(weight);
        self
    }

    pub fn clustering_weight(mut self, weight: usize) -> Self
    {
        self.clustering_weight = Some(weight);
        self
    }

    pub fn work_order_policies(mut self, path_to_work_order_policies: PathBuf) -> Result<Self>
    {
        let contents = std::fs::read_to_string(path_to_work_order_policies).unwrap();
        let work_order_policies: WorkOrderPolicies =
            toml::from_str(&contents).expect("Could not read WorkOrderPolicies");

        self.work_order_policies = Some(work_order_policies);
        Ok(self)
    }

    pub fn build(self) -> WeeklyOptions
    {
        WeeklyOptions {
            number_of_removed_work_orders: self
                .number_of_removed_work_orders
                .expect("number_of_removed_work_orders is required"),
            urgency_weight: self.urgency_weight.expect("urgency_weight is required"),
            resource_penalty_weight: self
                .resource_penalty_weight
                .expect("resource_penalty_weight is required"),
            clustering_weight: self
                .clustering_weight
                .expect("clustering_weight is required"),
            work_order_policies: self
                .work_order_policies
                .expect("This work order policies should be loaded"),
        }
    }
}

pub struct ProjectOptionsBuilder
{
    number_of_removed_work_orders: Option<usize>,
    urgency: Option<usize>,
    resource_penalty: Option<usize>,
}

impl ProjectOptionsBuilder
{
    pub fn new() -> Self
    {
        Self {
            number_of_removed_work_orders: None,
            urgency: None,
            resource_penalty: None,
        }
    }

    pub fn number_of_removed_work_orders(mut self, count: usize) -> Self
    {
        self.number_of_removed_work_orders = Some(count);
        self
    }

    pub fn urgency(mut self, urgency: usize) -> Self
    {
        self.urgency = Some(urgency);
        self
    }

    pub fn resource_penalty(mut self, penalty: usize) -> Self
    {
        self.resource_penalty = Some(penalty);
        self
    }

    pub fn build(self) -> ProjectOptions
    {
        ProjectOptions {
            number_of_removed_work_orders: self
                .number_of_removed_work_orders
                .expect("number_of_removed_work_orders is required"),
            urgency: self.urgency.expect("urgency is required"),
            resource_penalty: self.resource_penalty.expect("resource_penalty is required"),
        }
    }
}

pub struct DailyOptionsBuilder
{
    number_of_unassigned_work_orders: Option<usize>,
}

impl DailyOptionsBuilder
{
    pub fn new() -> Self
    {
        Self {
            number_of_unassigned_work_orders: None,
        }
    }

    pub fn number_of_unassigned_work_orders(mut self, count: usize) -> Self
    {
        self.number_of_unassigned_work_orders = Some(count);
        self
    }

    pub fn build(self) -> DailyOptions
    {
        DailyOptions {
            number_of_unassigned_work_orders: self
                .number_of_unassigned_work_orders
                .expect("number_of_unassigned_work_orders is required"),
        }
    }
}

pub struct OperationalOptionsBuilder
{
    number_of_removed_activities: Option<usize>,
}

impl OperationalOptionsBuilder
{
    pub fn new() -> Self
    {
        Self {
            number_of_removed_activities: None,
        }
    }

    pub fn number_of_removed_activities(mut self, count: usize) -> Self
    {
        self.number_of_removed_activities = Some(count);
        self
    }

    pub fn build(self) -> OperationalOptions
    {
        OperationalOptions {
            number_of_removed_activities: self
                .number_of_removed_activities
                .expect("number_of_removed_activities is required"),
        }
    }
}
#[cfg(test)]
mod tests
{

    // #[test]
    // fn test_toml_operational_parsing()
    // {
    //     let toml_operational_string = r#"
    //         [[dailys]]
    //         id = "main"
    //         number_of_supervisAgentEnvironmentr_periods = 3

    //         # [[dailys]]
    //         # id = "daily-second"
    //         ################################
    //         ###          MTN-ELEC        ###
    //         ################################
    //         [[operational]]
    //         id = "OP-01-001"
    //         resources.resources = ["MTN-ELEC" ]
    //         hours_per_day = 6.0
    //         operational_configuration.off_shift_interval = { start =
    // "19:00:00",  end = "07:00:00" }         operational_configuration.
    // break_interval = { start = "11:00:00", end = "12:00:00" }
    //         operational_configuration.toolbox_interval = { start =
    // "07:00:00", end = "08:00:00" }         operational_configuration.
    // availability.start_date = "2024-12-02T07:00:00Z"
    //         operational_configuration.availability.finish_date =
    // "2024-12-15T15:00:00Z"     "#;

    //     let system_agents: ActorSpecifications =
    // toml::from_str(toml_operational_string).unwrap();

    //     assert_eq!(system_agents.operational[0].id.0,
    // "OP-01-001".to_string());

    //     assert_eq!(system_agents.operational[0].id.1, [Resources::MtnElec]);

    use std::collections::HashMap;
    use std::str::FromStr;

    use chrono::NaiveDateTime;

    use crate::Asset;
    use crate::worker_environment::ActorSpecification;
    use crate::worker_environment::ActorSpecifications;
    use crate::worker_environment::IdString;
    use crate::worker_environment::InputOperational;
    use crate::worker_environment::availability::Availability;
    use crate::worker_environment::resources::ActorCompositeId;
    use crate::worker_environment::resources::Skill;

    #[test]
    fn test_actor_specification_builder()
    {
        let actor_spec = ActorSpecifications::builder()
            .weekly(|builder| {
                builder
                    .id("weekly-001")
                    .number_of_weekly_periods(5)
                    .weekly_options(|options| {
                        options
                            .number_of_removed_work_orders(10)
                            .urgency_weight(3)
                            .resource_penalty_weight(2)
                            .clustering_weight(1)
                    })
            })
            .project(|builder| {
                builder
                    .id("project-001")
                    .number_of_project_days(7)
                    .project_options(|options| {
                        options
                            .number_of_removed_work_orders(5)
                            .urgency(2)
                            .resource_penalty(1)
                    })
            })
            .dailys(|builder| {
                builder.daily(|sup| {
                    sup.id("daily-001")
                        .number_of_daily_periods(3)
                        .daily_options(|options| options.number_of_unassigned_work_orders(2))
                })
            })
            .build()
            .expect("Failed to build ActorSpecifications");

        assert_eq!(actor_spec.weekly.id, "weekly-001");
        assert_eq!(actor_spec.weekly.number_of_weekly_periods, 5);
        assert_eq!(actor_spec.project.id, "project-001");
        assert_eq!(actor_spec.project.number_of_project_days, 7);
        assert_eq!(actor_spec.dailys.len(), 1);
        assert_eq!(actor_spec.dailys[0].id, "daily-001");
    }

    #[test]
    fn test_add_technician()
    {
        // Mock dependencies for testing

        #[derive(Debug)]
        struct TestActorSpecification
        {
            operational: HashMap<IdString, InputOperational>,
        }

        impl ActorSpecification for TestActorSpecification
        {
            fn weekly_options(&self) -> &super::WeeklyOptions
            {
                todo!()
            }

            fn weekly(&self) -> &super::InputWeekly
            {
                todo!()
            }

            fn project(&self) -> &super::InputProject
            {
                todo!()
            }

            fn daily(&self) -> &Vec<super::InputDaily>
            {
                todo!()
            }

            fn operational(&self) -> &HashMap<IdString, InputOperational>
            {
                todo!()
            }

            fn technician_availability(
                &self,
            ) -> std::collections::BTreeMap<
                IdString,
                (
                    std::collections::BTreeSet<Availability>,
                    std::collections::HashSet<Skill>,
                ),
            >
            {
                todo!()
            }

            fn add_operational(
                &mut self,
                id: &IdString,
                assets: Vec<Asset>,
                resources: Vec<Skill>,
                start_date: chrono::DateTime<chrono::Utc>,
                finish_date: chrono::DateTime<chrono::Utc>,
            ) -> anyhow::Result<super::resources::ActorCompositeId>
            {
                let availability = Availability::new(start_date, finish_date, assets)?;

                self.operational
                    .entry(id.clone())
                    .and_modify(|e| {
                        e.operational_configuration
                            .availability
                            .insert(availability.clone());
                    })
                    .or_insert(InputOperational::new(
                        id.clone(),
                        resources.clone(),
                        6.0,
                        availability.clone(),
                    ));

                Ok(ActorCompositeId::new(id, resources, availability))
            }
        }

        let mut actor_specification = TestActorSpecification {
            operational: HashMap::new(),
        };

        let start_date = NaiveDateTime::from_str("2025-09-01T07:00:00")
            .unwrap()
            .and_utc();
        let finish_date = NaiveDateTime::from_str("2025-09-04T07:00:00")
            .unwrap()
            .and_utc();
        let assets = vec![Asset::Test];
        let id_string: IdString = "OP-01-test".to_string();
        let resources = vec![Skill::MtnLagg];

        actor_specification
            .add_operational(
                &id_string,
                assets.clone(),
                resources.clone(),
                start_date,
                finish_date,
            )
            .unwrap();

        assert!(actor_specification.operational.contains_key(&id_string));

        let availability_test_mock =
            Availability::new(start_date, finish_date, assets.clone()).unwrap();
        assert!(
            actor_specification
                .operational
                .get(&id_string)
                .unwrap()
                .operational_configuration
                .availability
                .contains(&availability_test_mock)
        );
        assert!(
            actor_specification
                .operational
                .get(&id_string)
                .unwrap()
                .operational_configuration
                .availability
                .len()
                == 1
        );

        let start_date_2 = NaiveDateTime::from_str("2025-09-06T07:00:00")
            .unwrap()
            .and_utc();
        let finish_date_2 = NaiveDateTime::from_str("2025-09-10T07:00:00")
            .unwrap()
            .and_utc();

        actor_specification
            .add_operational(
                &id_string,
                assets.clone(),
                resources,
                start_date_2,
                finish_date_2,
            )
            .unwrap();

        let availability_test_mock_2 =
            Availability::new(start_date_2, finish_date_2, assets.clone()).unwrap();
        assert!(
            actor_specification
                .operational
                .get(&id_string)
                .unwrap()
                .operational_configuration
                .availability
                .contains(&availability_test_mock_2)
        );
        assert!(
            actor_specification
                .operational
                .get(&id_string)
                .unwrap()
                .operational_configuration
                .availability
                .len()
                == 2
        );
    }
}
