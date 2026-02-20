use std::str::FromStr;

use clap::Args;
use clap::Subcommand;
use shared_types::agents::weekly::requests::weekly_request_resources_message::WeeklyRequestResource;
use shared_types::agents::weekly::requests::weekly_request_scheduling_message::ScheduleChange;
use shared_types::agents::weekly::requests::weekly_request_scheduling_message::WeeklyRequestScheduling;
use shared_types::agents::weekly::requests::weekly_request_status_message::WeeklyStatusMessage;
use shared_types::agents::weekly::WeeklyRequest;
use shared_types::agents::weekly::WeeklyRequestMessage;
use shared_types::agents::weekly::WeeklySchedulingEnvironmentCommands;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::Asset;
use shared_types::SystemMessages;

#[derive(Subcommand, Debug)]
pub enum WeeklyCommands
{
    /// Overview of the weekly agent
    Status
    {
        asset: Asset,
        #[clap(subcommand)]
        status_commands: Option<StatusCommands>,
    },
    /// Scheduling commands
    Scheduling
    {
        asset: Asset,
        #[clap(subcommand)]
        scheduling_commands: SchedulingCommands,
    },
    /// Resources commands
    Resources
    {
        asset: Asset,
        #[clap(subcommand)]
        resource_commands: ResourceCommands,
    },

    /// Access the Scheduling Environment with the options that the
    /// WeeklyAgent can change
    WeeklySchedulingEnvironmentCommands
    {
        asset: Asset,
        #[clap(subcommand)]
        weekly_scheduling_environment_commands: WeeklySchedulingEnvironmentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResourceCommands
{
    /// Get the loading of the resources
    Loading
    {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the capacity of the resources
    Capacity
    {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the percentage loading
    PercentageLoading
    {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },
    /// Set the capacity of a resource
    SetCapacity
    {
        /// Format YYYY-Wxx-xx (e.g. 2024-W41-42)
        period: String,
        resource: Vec<Resources>,
        capacity: f64,
    },
}

#[derive(Subcommand, Debug)]
pub enum StatusCommands
{
    /// List all work orders in a given period
    WorkOrders
    {
        period: String
    },
    /// List relevant information about a specific work order
    WorkOrder
    {
        work_order_number: u64
    },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingCommands
{
    /// Schedule a specific work order in a given period
    Schedule(ScheduleChange),
    /// Lock a period from any scheduling changes
    PeriodLock
    {
        period: String
    },
    /// Exclude a work order from a period
    Exclude(ScheduleChange),
}

#[derive(Debug, Args)]
pub struct WorkOrderSchedule
{
    pub work_order: u64,
    pub period: String,
}

impl WeeklyCommands
{
    pub fn execute(self) -> SystemMessages
    {
        match self {
            WeeklyCommands::Status {
                asset,
                status_commands,
            } => match status_commands {
                Some(StatusCommands::WorkOrder { work_order_number }) => {
                    let weekly_status_message =
                        WeeklyStatusMessage::WorkOrder(WorkOrderNumber(work_order_number));

                    let weekly_request = WeeklyRequest {
                        asset,
                        weekly_request_message: WeeklyRequestMessage::Status(
                            weekly_status_message,
                        ),
                    };

                    SystemMessages::Weekly(weekly_request)
                }
                Some(StatusCommands::WorkOrders { period }) => {
                    let weekly_status_message =
                        WeeklyStatusMessage::new_period(period.to_string());

                    let weekly_request = WeeklyRequest {
                        asset,
                        weekly_request_message: WeeklyRequestMessage::Status(
                            weekly_status_message,
                        ),
                    };

                    SystemMessages::Weekly(weekly_request)
                }
                None => {
                    let weekly_status_message: WeeklyStatusMessage =
                        WeeklyStatusMessage::General;

                    let weekly_request_message =
                        WeeklyRequestMessage::Status(weekly_status_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }
            },
            WeeklyCommands::Scheduling {
                asset,
                scheduling_commands: subcommand,
            } => match subcommand {
                SchedulingCommands::Schedule(schedule) => {
                    let weekly_scheduling_message: WeeklyRequestScheduling =
                        WeeklyRequestScheduling::Schedule(schedule);

                    let weekly_request_message =
                        WeeklyRequestMessage::Scheduling(weekly_scheduling_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }
                SchedulingCommands::PeriodLock { period: _ } => {
                    todo!()
                }
                SchedulingCommands::Exclude(schedule_change) => {
                    let weekly_scheduling_message: WeeklyRequestScheduling =
                        WeeklyRequestScheduling::ExcludeFromPeriod(schedule_change);

                    let weekly_request_message =
                        WeeklyRequestMessage::Scheduling(weekly_scheduling_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }
            },
            WeeklyCommands::Resources {
                asset,
                resource_commands: subcommand,
            } => match subcommand {
                ResourceCommands::Loading {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let weekly_resources_message = WeeklyRequestResource::GetLoadings {
                        periods_end: periods_end.to_string(),
                        select_resources: resources,
                    };

                    let weekly_request_message =
                        WeeklyRequestMessage::Resources(weekly_resources_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }
                ResourceCommands::Capacity {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let weekly_resources_message = WeeklyRequestResource::GetCapacities {
                        periods_end: periods_end.to_string(),
                        select_resources: resources,
                    };

                    let weekly_request_message =
                        WeeklyRequestMessage::Resources(weekly_resources_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }

                ResourceCommands::PercentageLoading {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let weekly_resources_message =
                        WeeklyRequestResource::GetPercentageLoadings {
                            periods_end: periods_end.to_string(),
                            resources,
                        };

                    let weekly_request_message =
                        WeeklyRequestMessage::Resources(weekly_resources_message);

                    let weekly_request = WeeklyRequest {
                        asset: asset.clone(),
                        weekly_request_message,
                    };

                    SystemMessages::Weekly(weekly_request)
                }
                ResourceCommands::SetCapacity {
                    resource: _,
                    period: _,
                    capacity: _,
                } => {
                    todo!()
                }
            },
            WeeklyCommands::WeeklySchedulingEnvironmentCommands {
                asset,
                weekly_scheduling_environment_commands,
            } => {
                let weekly_request_message = WeeklyRequestMessage::SchedulingEnvironment(
                    weekly_scheduling_environment_commands,
                );

                let weekly_request = WeeklyRequest {
                    asset,
                    weekly_request_message,
                };
                SystemMessages::Weekly(weekly_request)
            }
        }
    }
}
