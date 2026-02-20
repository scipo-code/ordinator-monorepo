use std::io::Read;

use clap::Args;
use clap::Subcommand;
use reqwest::blocking::Client;
use shared_types::agents::daily::requests::daily_scheduling_message::DailySchedulingMessage;
use shared_types::agents::daily::requests::daily_status_message::DailyStatusMessage;
use shared_types::agents::daily::DailyRequest;
use shared_types::agents::daily::DailyRequestMessage;
use shared_types::agents::daily::DailyType;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::Asset;
use shared_types::SystemMessages;

#[derive(Subcommand, Debug)]
pub enum DailyCommands
{
    /// Get the status of a DailyAgent
    Status
    {
        asset: Asset,
        daily: DailyType,
    },
    /// Get the commands for manually scheduling a work order activity
    Scheduling
    {
        asset: Asset,
        daily_type: DailyType,
        #[clap(subcommand)]
        scheduling_commands: SchedulingCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingCommands
{
    /// Schedule a specific work order activity to an operational agent
    Schedule(Assign),
}

#[derive(Args, Debug)]
pub struct Assign
{
    work_order_number: u64,
    activity_number: u64,
    id_operational: String,
}

impl DailyCommands
{
    pub fn execute(&self, client: &Client) -> SystemMessages
    {
        match self {
            DailyCommands::Status { asset, daily } => {
                let daily_status_message = DailyStatusMessage::General;

                let daily_request_message =
                    DailyRequestMessage::Status(daily_status_message);

                let daily_request = DailyRequest {
                    asset: asset.clone(),
                    daily: daily.clone(),
                    daily_request_message,
                };

                SystemMessages::Daily(daily_request)
            }
            DailyCommands::Scheduling {
                asset,
                daily_type,
                scheduling_commands,
            } => match scheduling_commands {
                SchedulingCommands::Schedule(assign) => {
                    let id_operational = get_id_operational(client, assign.id_operational.clone());

                    let daily_scheduling_message = DailySchedulingMessage::new(
                        (assign.work_order_number.into(), assign.activity_number),
                        id_operational,
                    );

                    let daily_request_message =
                        DailyRequestMessage::Scheduling(daily_scheduling_message);

                    let daily_request = DailyRequest {
                        asset: asset.clone(),
                        daily: daily_type.clone(),
                        daily_request_message,
                    };

                    SystemMessages::Daily(daily_request)
                }
            },
        }
    }
}

fn get_id_operational(client: &Client, id_operational: String) -> Id
{
    let url: String = "http://".to_string()
        + &dotenvy::var("IMPERIUM_ADDRESS").unwrap()
        + &dotenvy::var("ORDINATOR_MAIN_ENDPOINT)").unwrap();

    let mut id_operational_json = String::new();
    client
        .get(url)
        .body(id_operational)
        .send()
        .unwrap()
        .read_to_string(&mut id_operational_json)
        .unwrap();

    let id_operational: Id = serde_json::from_str(&id_operational_json).unwrap();
    id_operational
}
