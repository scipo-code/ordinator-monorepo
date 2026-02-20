use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use shared_types::agents::project::requests::project_resources_message::ProjectResourceRequest;
use shared_types::agents::project::requests::project_status_message::ProjectStatusMessage;
use shared_types::agents::project::Days;
use shared_types::agents::project::ProjectRequest;
use shared_types::agents::project::ProjectRequestMessage;
use shared_types::agents::project::ProjectResources;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::ActorSpecifications;
use shared_types::Asset;
use shared_types::SystemMessages;

use super::orchestrator;

#[derive(Subcommand, Debug)]
pub enum ProjectCommands
{
    /// Get the status of the project agent
    Status
    {
        asset: Asset
    },
    /// Get the objectives of the project agent
    Resources
    {
        asset: Asset,
        #[clap(subcommand)]
        resource_commands: ResourceCommands,
    },
    /// Access the scheduling of the project agent
    Scheduling,
    /// Access the days of the project agent
    Days,
}

impl ProjectCommands
{
    pub fn execute(&self, client: &Client) -> shared_types::SystemMessages
    {
        match self {
            ProjectCommands::Status { asset } => {
                let project_request = ProjectRequest {
                    asset: asset.clone(),
                    project_request_message: ProjectRequestMessage::Status(
                        ProjectStatusMessage::General,
                    ),
                };

                SystemMessages::Project(project_request)
            }

            ProjectCommands::Resources {
                asset,
                resource_commands,
            } => match resource_commands {
                ResourceCommands::Capacity {
                    days_end,
                    select_resources,
                } => {
                    let project_resources_message = ProjectResourceRequest::GetCapacities {
                        days_end: days_end.to_string(),
                        select_resources: select_resources.clone(),
                    };

                    let project_request_request =
                        ProjectRequestMessage::Resources(project_resources_message);

                    let project_request = ProjectRequest {
                        asset: asset.clone(),
                        project_request_message: project_request_request,
                    };

                    SystemMessages::Project(project_request)
                }
                ResourceCommands::Loading {
                    days_end,
                    select_resources,
                } => {
                    let project_resources_message = ProjectResourceRequest::GetLoadings {
                        days_end: days_end.to_string(),
                        select_resources: select_resources.clone(),
                    };

                    let project_request_message =
                        ProjectRequestMessage::Resources(project_resources_message);

                    let project_request = ProjectRequest {
                        asset: asset.clone(),
                        project_request_message,
                    };

                    SystemMessages::Project(project_request)
                }
                ResourceCommands::PercentageLoading {
                    days_end,
                    select_resources,
                } => {
                    let project_resources_message =
                        ProjectResourceRequest::GetPercentageLoadings {
                            days_end: days_end.to_string(),
                            resources: select_resources.clone(),
                        };

                    let project_request_message =
                        ProjectRequestMessage::Resources(project_resources_message);

                    let project_request = ProjectRequest {
                        asset: asset.clone(),
                        project_request_message,
                    };
                    SystemMessages::Project(project_request)
                }
                ResourceCommands::LoadCapacityFile { toml_path } => {
                    let resources = generate_manual_resources(client, toml_path.clone());

                    let project_resources = resources;
                    let project_resources_message =
                        ProjectResourceRequest::new_set_resources(project_resources);

                    let project_request_message =
                        ProjectRequestMessage::Resources(project_resources_message);
                    let project_request = ProjectRequest {
                        asset: asset.clone(),
                        project_request_message,
                    };
                    SystemMessages::Project(project_request)
                }
            },
            ProjectCommands::Scheduling => {
                todo!()
            }
            ProjectCommands::Days => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ResourceCommands
{
    Loading
    {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    Capacity
    {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    PercentageLoading
    {
        days_end: u32,
        select_resources: Option<Vec<Resources>>,
    },
    /// Set a capacity based on a file
    LoadCapacityFile
    {
        toml_path: String
    },
}

/// Generates manual resources for the project agent from a TOML configuration file.
fn generate_manual_resources(client: &Client, toml_path: String) -> ProjectResources
{
    let days: Vec<Day> = orchestrator::project_days(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();

    let config: ActorSpecifications = toml::from_str(&contents).unwrap();

    let _hours_per_day = 6.0;

    let gradual_reduction = |i: usize| -> f64 {
        match i {
            0..=13 => 1.0,
            14..=27 => 1.0,
            _ => 1.0,
        }
    };

    let mut resources_hash_map = HashMap::<Resources, Days>::new();
    for operational_agent in config.operational {
        for (i, day) in days.clone().iter().enumerate() {
            let resource_periods = resources_hash_map
                .entry(operational_agent.resources.first().cloned().unwrap())
                .or_insert(Days::new(HashMap::new()));

            *resource_periods.days.entry(day.clone()).or_insert_with(|| {
                Work::from(operational_agent.hours_per_day * gradual_reduction(i))
            }) += Work::from(operational_agent.hours_per_day * gradual_reduction(i))
        }
    }
    ProjectResources::new(resources_hash_map)
}
