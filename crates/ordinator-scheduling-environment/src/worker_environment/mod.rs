pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
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
use crate::time_environment::MaterialToPeriod;
use crate::time_environment::TimeInterval;
use crate::work_order::WorkOrderConfigurations;

pub type OperationalId = String;
// There is something rotten about all this! I think that the best
// approach is to create something that will allow us to better
// forcast how the system will behave.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ActorEnvironment
{
    // I think that the actor environment is the correct term here.
    // Changes to the parameters should be changable in the application
    // itself. Where does that leave all this. Maybe we should actually
    // just make the... I think that we would require to make the. There
    // will be required some extreme logic here.
    pub actor_specification: HashMap<Asset, ActorSpecifications>,
}

pub struct ActorEnvironmentBuilder
{
    pub actor_environment: HashMap<Asset, ActorSpecifications>,
}

impl ActorEnvironment
{
    // TODO [ ]
    // This should be refactored!
    pub fn builder() -> ActorEnvironmentBuilder
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

impl ActorEnvironmentBuilder
{
    pub fn build(self) -> ActorEnvironment
    {
        ActorEnvironment {
            actor_specification: self.actor_environment,
        }
    }

    // We should insert... This builder is a little bothersome.
    // Ideally we need to provide a resource file for each of the different.
    // assets. That means that this should be callable many times over for
    // this to work.
    pub fn actor_environment(mut self, asset: Asset, path_to_data: PathBuf) -> Result<Self>
    {
        println!("{}", std::env::current_dir()?.display());

        // This should then be changed into something different for this to
        // work. You need to put it into the Asset and the ... I think that
        // it is okay to simply hard code the information for now. Hmm...
        // The issues comes from the difference between using the toml file
        // for initialization and using it for data storage... I think that
        // for now you should simply follow the same model that is used in
        // for the work orders: If the database file is missing you should
        // perform a complete reinitialization of the system. And if not
        // you should simply use the JSON file.
        //
        // For now the most important thing is getting all the data into the
        // `SchedulingEnvironment`
        // WARN This should not be needed to solve this problem. Keep it for now
        // DATE 2025-05-01
        // let list_of_actor_specification = vec![
        //     (
        //
        //         Asset::DF,
        //         "./configuration/actor_specification/actor_specification_df.toml",
        //     ),
        //     (
        //         Asset::HB,
        //         "./configuration/actor_specification/actor_specification_hb.toml",
        //     ),
        //     (
        //         Asset::HD,
        //         "./configuration/actor_specification/actor_specification_hd.toml",
        //     ),
        //     (
        //         Asset::Test,

        //         "./configuration/actor_specification/actor_specification_test.toml",
        //     ),
        //     (
        //         Asset::TE,
        //         "./configuration/actor_specification/actor_specification_te.toml",
        //     ),
        // ];

        // You should put the data into the toml? Yes I think that is the best approach
        // here.

        let contents = std::fs::read_to_string(&path_to_data).with_context(|| {
            format!(
                "Could not read string for ActorSpecification\nPath: {}",
                path_to_data.display()
            )
        })?;

        let actor_specifications: ActorSpecifications =
            toml::from_str(&contents).with_context(|| {
                format!(
                    "Could not deserialize into ActorSpecification. File: {}, Line: {}\nContent String\n{contents}",
                    file!(),
                    line!()
                )
            })?;

        self.actor_environment.insert(asset, actor_specifications);
        Ok(self)
    }
}

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
    pub work_order_configurations: WorkOrderConfigurations,
    pub material_to_period: MaterialToPeriod,
}

impl ActorSpecifications
{
    pub fn add_operational(
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

    pub fn technician_availability(
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

    use super::ActorSpecifications;
    use crate::Asset;
    use crate::worker_environment::IdString;
    use crate::worker_environment::availability::Availability;
    use crate::worker_environment::resources::Resources;

    #[test]
    fn test_add_technician()
    {
        // You have to mock all these dependencies to test the code.

        let mut actor_specification = ActorSpecifications {
            strategic: todo!(),
            tactical: todo!(),
            supervisors: vec![],
            operational: HashMap::new(),
            work_order_configurations: crate::work_order::WorkOrderConfigurations {
                order_type_weights: HashMap::new(),
                status_weights: HashMap::new(),
                vis_priority_map: HashMap::new(),
                wdf_priority_map: HashMap::new(),
                wgn_priority_map: HashMap::new(),
                wpm_priority_map: HashMap::new(),
                clustering_weights: crate::work_order::ClusteringWeights {
                    asset: 0,
                    sector: 0,
                    system: 0,
                    subsystem: 0,
                    equipment_tag: 0,
                },
                operating_time: 6,
            },
            material_to_period: crate::time_environment::MaterialToPeriod {
                nmat: 0,
                smat: 0,
                cmat: 0,
                pmat: 0,
                wmat: 0,
            },
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
            .add_operational(&id_string, assets, resources, start_date, finish_date)
            .unwrap();

        assert!(actor_specification.operational.contains_key(&id_string));

        let availability_test_mock = Availability::new(start_date, finish_date, assets).unwrap();
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
        let finish_date_2 = NaiveDateTime::from_str("2025-09-010T07:00:00")
            .unwrap()
            .and_utc();

        actor_specification
            .add_operational(&id_string, assets, resources, start_date_2, finish_date_2)
            .unwrap();

        let availability_test_mock_2 =
            Availability::new(start_date_2, finish_date_2, assets).unwrap();
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
