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
use chrono::NaiveTime;
use chrono::Utc;
use crew::OperationalConfiguration;
use crew::OperationalConfigurationBuilder;
use resources::ActorCompositeId;
use resources::Resources;
use serde::Deserialize;
use serde::Serialize;

use crate::Asset;
use crate::time_environment::TimeInterval;

pub type OperationalId = String;

// ESSAY:
//
// There is something rotten about all this! I think that the best
// approach is to create something that will allow us to better
// forcast how the system will behave.
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
    // TODO [ ]
    // This should be refactored!
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
    fn strategic_options(&self) -> &StrategicOptions;

    fn strategic(&self) -> &InputStrategic;

    fn tactical(&self) -> &InputTactical;

    fn supervisor(&self) -> &Vec<InputSupervisor>;

    fn operational(&self) -> &HashMap<IdString, InputOperational>;

    // let lock = orchestrator.actor_registries.lock().unwrap();
    // let asset = Asset::try_from(asset).map_err(|e|
    // AppError::Anyhow(e.to_string()))?;

    fn technician_availability(
        &self,
    ) -> BTreeMap<IdString, (BTreeSet<Availability>, HashSet<Resources>)>;

    fn add_operational(
        &mut self,
        id: &IdString,
        assets: Vec<Asset>,
        resources: Vec<Resources>,
        start_date: DateTime<Utc>,
        finish_date: DateTime<Utc>,
        // This should return a
    ) -> anyhow::Result<ActorCompositeId>;
}

//
// ISSUE #004 [ ] - make a trait implementation for this.
pub type IdString = String;
#[derive(Serialize, Deserialize, Debug)]
pub struct ActorSpecifications
{
    pub strategic: InputStrategic,
    pub tactical: InputTactical,
    pub supervisors: Vec<InputSupervisor>,
    pub operational: HashMap<IdString, InputOperational>,
    // QUESTION [x] Is this the way to do it?
    // It cannot be like this. The idea of a relational database is beginning
    // to make a lot of sense.
}

impl ActorSpecification for ActorSpecifications
{
    fn operational(&self) -> &HashMap<IdString, InputOperational>
    {
        &self.operational
    }

    fn strategic_options(&self) -> &StrategicOptions
    {
        &self.strategic.strategic_options
    }

    fn supervisor(&self) -> &Vec<InputSupervisor>
    {
        &self.supervisors
    }

    fn tactical(&self) -> &InputTactical
    {
        &self.tactical
    }

    fn strategic(&self) -> &InputStrategic
    {
        &self.strategic
    }

    fn technician_availability(
        &self,
    ) -> BTreeMap<IdString, (BTreeSet<Availability>, HashSet<Resources>)>
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
        resources: Vec<Resources>,
        start_date: DateTime<Utc>,
        finish_date: DateTime<Utc>,
        // This should return a
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
    strategic: Option<InputStrategic>,
    tactical: Option<InputTactical>,
    supervisors: Option<Vec<InputSupervisor>>,
    operational: Option<HashMap<IdString, InputOperational>>,
}

impl ActorSpecificationBuilder
{
    pub fn new() -> Self
    {
        Self {
            strategic: None,
            tactical: None,
            supervisors: None,
            operational: None,
        }
    }

    pub fn strategic<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputStrategicBuilder) -> InputStrategicBuilder,
    {
        let strategic_builder = InputStrategic::builder();
        let strategic_builder = f(strategic_builder);
        self.strategic = Some(strategic_builder.build());
        self
    }

    pub fn tactical<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputTacticalBuilder) -> InputTacticalBuilder,
    {
        let tactical_builder = InputTactical::builder();
        let tactical_builder = f(tactical_builder);
        self.tactical = Some(tactical_builder.build());
        self
    }

    pub fn supervisors<F>(mut self, f: F) -> Self
    where
        F: FnOnce(SupervisorsBuilder) -> SupervisorsBuilder,
    {
        let supervisors_builder = SupervisorsBuilder::new();
        let supervisors_builder = f(supervisors_builder);
        self.supervisors = Some(supervisors_builder.build());
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
            strategic: self
                .strategic
                .ok_or_else(|| anyhow::anyhow!("Strategic configuration is required"))?,
            tactical: self
                .tactical
                .ok_or_else(|| anyhow::anyhow!("Tactical configuration is required"))?,
            supervisors: self.supervisors.unwrap_or_default(),
            operational: self.operational.unwrap_or_default(),
        })
    }
}

// This should be handled as well. What should you do not? I think that a
// meditation session.
//

// TODO #00 #00 #03 [x] Move the `./configuration/work_order_parameters.json`
// here. Is this
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputStrategic
{
    pub id: IdString,
    pub number_of_strategic_periods: usize,
    pub strategic_options: StrategicOptions,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputTactical
{
    pub id: IdString,
    pub number_of_tactical_days: usize,
    pub tactical_options: TacticalOptions,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputSupervisor
{
    pub id: IdString,
    pub number_of_supervisor_periods: u64,
    pub supervisor_options: SupervisorOptions,
}

// TODO [ ]
// Load in the IDs directly.
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
        resources: Vec<Resources>,
        hours_per_day: f64,

        availability: Availability,
    ) -> Self
    {
        let resources = resources.iter().cloned().collect::<HashSet<_>>();
        let availabilities = BTreeSet::from([availability]);
        let operational_configuration = OperationalConfiguration::new(
            availabilities,
            TimeInterval {
                start: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            },
            TimeInterval {
                start: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            },
            TimeInterval {
                start: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            },
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
/// This type is for loading in the `Strategic` configurations
/// so that the `StrategicOptions` can be loaded in to the `Agent`
/// in the correct format.
/// How to resolve this duplication? Do you want this in the database?
/// So you have already understood that this is the case. This is the
/// priority that you need to understand here.
// QUESTION [ ]
// What should you do about the `StdRng`? I think that the best approach
// here is to make the code. You have to make you own Deser
//
// It should leave. The five why was essential. Leave the code out of this. I think
// that the correct way of making this is the the Orchestrator should apply changes
//
// QUESTION [ ]
// So the key question here is whether the Actor will ever need to
// see the options? I do not believe that it is. Actuallu does the
// [`Orchestrator`] even need to know the Actors?
//
// The issue here is that you are afraid of using `dyn`. That is the
// main thing that you need to have more decoupling.
//
// This has to be Clone. Otherwise you will not be able to understand the
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct StrategicOptions
{
    pub number_of_removed_work_orders: usize,
    pub urgency_weight: usize,
    pub resource_penalty_weight: usize,
    pub clustering_weight: usize,
    // These two should go into the `SchedulingEnvironment` that means that
    // the code should strive to... This means that the StrategicAgent, would
    // simply import this directly into itself. There is no need for a
    //
    // You can move this directly from the scheduling environment into the
    // Actor. Is the a good idea? I think that it is
    // The priority should be the same for each work order, correct? Yes
    // I think that is the correct answer.
}

// The `rng` should not be inside of the `ordinator-scheduling-environment`
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct TacticalOptions
{
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorOptions
{
    pub number_of_unassigned_work_orders: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct OperationalOptions
{
    pub number_of_removed_activities: usize,
}

pub struct InputStrategicBuilder
{
    id: Option<IdString>,
    number_of_strategic_periods: Option<usize>,
    strategic_options: Option<StrategicOptions>,
}

impl InputStrategicBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_strategic_periods: None,
            strategic_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_strategic_periods(mut self, periods: usize) -> Self
    {
        self.number_of_strategic_periods = Some(periods);
        self
    }

    pub fn strategic_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(StrategicOptionsBuilder) -> StrategicOptionsBuilder,
    {
        let options_builder = StrategicOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.strategic_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputStrategic
    {
        InputStrategic {
            id: self.id.expect("id is required"),
            number_of_strategic_periods: self
                .number_of_strategic_periods
                .expect("number_of_strategic_periods is required"),
            strategic_options: self
                .strategic_options
                .expect("strategic_options is required"),
        }
    }
}

impl InputStrategic
{
    pub fn builder() -> InputStrategicBuilder
    {
        InputStrategicBuilder::new()
    }
}

pub struct InputTacticalBuilder
{
    id: Option<IdString>,
    number_of_tactical_days: Option<usize>,
    tactical_options: Option<TacticalOptions>,
}

impl InputTacticalBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_tactical_days: None,
            tactical_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_tactical_days(mut self, days: usize) -> Self
    {
        self.number_of_tactical_days = Some(days);
        self
    }

    pub fn tactical_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TacticalOptionsBuilder) -> TacticalOptionsBuilder,
    {
        let options_builder = TacticalOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.tactical_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputTactical
    {
        InputTactical {
            id: self.id.expect("id is required"),
            number_of_tactical_days: self
                .number_of_tactical_days
                .expect("number_of_tactical_days is required"),
            tactical_options: self.tactical_options.expect("tactical_options is required"),
        }
    }
}

impl InputTactical
{
    pub fn builder() -> InputTacticalBuilder
    {
        InputTacticalBuilder::new()
    }
}

pub struct SupervisorsBuilder
{
    supervisors: Vec<InputSupervisor>,
}

impl SupervisorsBuilder
{
    pub fn new() -> Self
    {
        Self {
            supervisors: Vec::new(),
        }
    }

    pub fn supervisor<F>(mut self, f: F) -> Self
    where
        F: FnOnce(InputSupervisorBuilder) -> InputSupervisorBuilder,
    {
        let supervisor_builder = InputSupervisorBuilder::new();
        let supervisor_builder = f(supervisor_builder);
        self.supervisors.push(supervisor_builder.build());
        self
    }

    pub fn build(self) -> Vec<InputSupervisor>
    {
        self.supervisors
    }
}

pub struct InputSupervisorBuilder
{
    id: Option<IdString>,
    number_of_supervisor_periods: Option<u64>,
    supervisor_options: Option<SupervisorOptions>,
}

impl Default for InputSupervisorBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl InputSupervisorBuilder
{
    pub fn new() -> Self
    {
        Self {
            id: None,
            number_of_supervisor_periods: None,
            supervisor_options: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self
    {
        self.id = Some(id.to_string());
        self
    }

    pub fn number_of_supervisor_periods(mut self, periods: u64) -> Self
    {
        self.number_of_supervisor_periods = Some(periods);
        self
    }

    pub fn supervisor_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce(SupervisorOptionsBuilder) -> SupervisorOptionsBuilder,
    {
        let options_builder = SupervisorOptionsBuilder::new();
        let options_builder = f(options_builder);
        self.supervisor_options = Some(options_builder.build());
        self
    }

    pub fn build(self) -> InputSupervisor
    {
        InputSupervisor {
            id: self.id.expect("id is required"),
            number_of_supervisor_periods: self
                .number_of_supervisor_periods
                .expect("number_of_supervisor_periods is required"),
            supervisor_options: self
                .supervisor_options
                .expect("supervisor_options is required"),
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

pub struct StrategicOptionsBuilder
{
    number_of_removed_work_orders: Option<usize>,
    urgency_weight: Option<usize>,
    resource_penalty_weight: Option<usize>,
    clustering_weight: Option<usize>,
}

impl StrategicOptionsBuilder
{
    pub fn new() -> Self
    {
        Self {
            number_of_removed_work_orders: None,
            urgency_weight: None,
            resource_penalty_weight: None,
            clustering_weight: None,
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

    pub fn build(self) -> StrategicOptions
    {
        StrategicOptions {
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
        }
    }
}

pub struct TacticalOptionsBuilder
{
    number_of_removed_work_orders: Option<usize>,
    urgency: Option<usize>,
    resource_penalty: Option<usize>,
}

impl TacticalOptionsBuilder
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

    pub fn build(self) -> TacticalOptions
    {
        TacticalOptions {
            number_of_removed_work_orders: self
                .number_of_removed_work_orders
                .expect("number_of_removed_work_orders is required"),
            urgency: self.urgency.expect("urgency is required"),
            resource_penalty: self.resource_penalty.expect("resource_penalty is required"),
        }
    }
}

pub struct SupervisorOptionsBuilder
{
    number_of_unassigned_work_orders: Option<usize>,
}

impl SupervisorOptionsBuilder
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

    pub fn build(self) -> SupervisorOptions
    {
        SupervisorOptions {
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
    //         [[supervisors]]
    //         id = "main"
    //         number_of_supervisAgentEnvironmentr_periods = 3

    //         # [[supervisors]]
    //         # id = "supervisor-second"
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
    //     assert_eq!(
    //         system_agents.operational[0]
    //             .operational_configuration
    //             .off_shift_interval
    //             .start,
    //         NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
    //     );
    //     assert_eq!(
    //         system_agents.operational[0]
    //             .operational_configuration
    //             .off_shift_interval
    //             .end,
    //         NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
    //     );
    // }
    use std::str::FromStr;

    use chrono::NaiveDateTime;

    use crate::Asset;
    use crate::worker_environment::ActorSpecification;
    use crate::worker_environment::ActorSpecifications;
    use crate::worker_environment::IdString;
    use crate::worker_environment::InputOperational;
    use crate::worker_environment::availability::Availability;
    use crate::worker_environment::resources::ActorCompositeId;
    use crate::worker_environment::resources::Resources;

    //
    #[test]
    fn test_actor_specification_builder()
    {
        let actor_spec = ActorSpecifications::builder()
            .strategic(|builder| {
                builder
                    .id("strategic-001")
                    .number_of_strategic_periods(5)
                    .strategic_options(|options| {
                        options
                            .number_of_removed_work_orders(10)
                            .urgency_weight(3)
                            .resource_penalty_weight(2)
                            .clustering_weight(1)
                    })
            })
            .tactical(|builder| {
                builder
                    .id("tactical-001")
                    .number_of_tactical_days(7)
                    .tactical_options(|options| {
                        options
                            .number_of_removed_work_orders(5)
                            .urgency(2)
                            .resource_penalty(1)
                    })
            })
            .supervisors(|builder| {
                builder.supervisor(|sup| {
                    sup.id("supervisor-001")
                        .number_of_supervisor_periods(3)
                        .supervisor_options(|options| options.number_of_unassigned_work_orders(2))
                })
            })
            .build()
            .expect("Failed to build ActorSpecifications");

        assert_eq!(actor_spec.strategic.id, "strategic-001");
        assert_eq!(actor_spec.strategic.number_of_strategic_periods, 5);
        assert_eq!(actor_spec.tactical.id, "tactical-001");
        assert_eq!(actor_spec.tactical.number_of_tactical_days, 7);
        assert_eq!(actor_spec.supervisors.len(), 1);
        assert_eq!(actor_spec.supervisors[0].id, "supervisor-001");
    }

    #[test]
    fn test_add_technician()
    {
        // You have to mock all these dependencies to test the code.

        #[derive(Debug)]
        struct TestActorSpecification
        {
            operational: HashMap<IdString, InputOperational>,
        }

        impl ActorSpecification for TestActorSpecification
        {
            fn strategic_options(&self) -> &super::StrategicOptions
            {
                todo!()
            }

            fn strategic(&self) -> &super::InputStrategic
            {
                todo!()
            }

            fn tactical(&self) -> &super::InputTactical
            {
                todo!()
            }

            fn supervisor(&self) -> &Vec<super::InputSupervisor>
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
                    std::collections::HashSet<Resources>,
                ),
            >
            {
                todo!()
            }

            fn add_operational(
                &mut self,
                id: &IdString,
                assets: Vec<Asset>,
                resources: Vec<Resources>,
                start_date: chrono::DateTime<chrono::Utc>,
                finish_date: chrono::DateTime<chrono::Utc>,
                // This should return a
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
        let resources = vec![Resources::MtnLagg];

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
