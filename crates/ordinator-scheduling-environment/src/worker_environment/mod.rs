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
    use crate::worker_environment::IdString;
    use crate::worker_environment::InputOperational;
    use crate::worker_environment::availability::Availability;
    use crate::worker_environment::resources::ActorCompositeId;
    use crate::worker_environment::resources::Resources;

    //
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
