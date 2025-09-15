mod material;

pub mod throttling;
pub mod time_input;
pub mod toml_baptiste;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use ordinator_scheduling_environment::SystemConfigurationTrait;
use throttling::Throttling;
use toml_baptiste::BaptisteToml;

// QUESTION
// How should this be handled?
// They should be handled by created by handling a `From<<Actor>OptionConfig>
// for <Actor>Config` in the `ordinator-actors` crate!

/// This struct is used to load in all configuraions centrally into the
/// Orchestrator. The `Orchestrator` then uses dependency injection to provide
/// the actors with the correct `Configurations`.
// There is something that you do not understand here. Where should
// all these configurations go?
// WARN
// Remember! You have a single source of all configurations here,
// so there is no reason to question that in the system.
#[derive(Debug)]
pub struct SystemConfigurations
{
    pub data_locations: BaptisteToml,
    pub throttling: Throttling,
    pub temp_database_path: PathBuf,
}

impl SystemConfigurationTrait for SystemConfigurations {}

// FIX [ ]
// This is a good initial approach but remember to make it better if you have to
// revisit it.
// It has now come to making it better!
// There is something completely wrong here.
impl SystemConfigurations
{
    pub fn read_all_configs() -> Result<Arc<ArcSwap<SystemConfigurations>>>
    {
        let baptiste_data_locations_contents =
            std::fs::read_to_string("./configuration/data_locations/baptiste_data_locations.toml")
                .context("Could not find data files for manual SAP work order input\n\t* Are you in a test environment?")?;
        let data_locations = toml::from_str(&baptiste_data_locations_contents)
            .context("Could not deserialize the `BaptisteToml`")?;

        let throttling_contents =
            std::fs::read_to_string("./configuration/throttling/throttling.toml")
                .context("Could not find the `Throttling` configuration file")?;
        let throttling: Throttling = toml::from_str(&throttling_contents)
            .context("Could not deserialize the `Throttling` configuration")?;

        let database_path_string =
            &dotenvy::var("WORK_ORDERS_PATH").context("Could not read database path")?;

        let database_path = std::path::Path::new(database_path_string);

        // I believe that it is the best appraoch here to make sure that the
        // `Configurations` are always created wrapped. Then you will never
        // make the mistake, of accessing wild and stray configurations.
        Ok(Arc::new(ArcSwap::new(Arc::new(SystemConfigurations {
            data_locations,
            throttling,
            temp_database_path: database_path.to_owned(),
        }))))
    }

    pub fn build_configs(throttling: Throttling) -> Arc<ArcSwap<SystemConfigurations>>
    {
        // I believe that it is the best appraoch here to make sure that the
        // `Configurations` are always created wrapped. Then you will never
        // make the mistake, of accessing wild and stray configurations.
        Arc::new(ArcSwap::new(Arc::new(SystemConfigurations {
            data_locations: BaptisteToml::default(),
            throttling,
            temp_database_path: std::path::Path::new("NOT NEEDED IN TESTING").to_owned(),
        })))
    }

    // This is actually a `From <SystemConfiguration> for StrateticOptions`
}

// This should be a part of the creation of the `SchedulingEnvironment`
// #[test]
// fn test_read_config() {
//     let system_configurations =
// SystemConfigurations::read_all_configs().unwrap();

//     println!("{:#?}", system_configurations);
// }
