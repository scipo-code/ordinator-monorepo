use clap::Subcommand;

pub mod operational;
pub mod orchestrator;
pub mod sap;
pub mod status;
pub mod strategic;
pub mod supervisor;
pub mod tactical;

use orchestrator::OrchestratorCommands;
use reqwest::blocking::Client;
use sap::SapCommands;
use shared_types::orchestrator::OrchestratorRequest;
use shared_types::Asset;
use shared_types::SystemMessages;
use status::StatusCommands;
use status::WorkOrders;
use strategic::WeeklyCommands;
use tactical::ProjectCommands;

use self::operational::OperationalCommands;
use self::supervisor::DailyCommands;
use crate::Cli;

/// Imperium: The command line interface to access the Ordinator scheduling
/// system API. All commands return a JSON unless otherwise specified.
#[derive(Subcommand)]
pub enum Commands
{
    /// Access general status command for imperium
    Status
    {
        #[clap(subcommand)]
        status_commands: StatusCommands,
    },
    /// Access the orchestrator agent (controls the scheduling environment)
    Orchestrator
    {
        #[clap(subcommand)]
        orchestrator_commands: OrchestratorCommands,
    },
    /// Access the strategic agent
    Weekly
    {
        #[clap(subcommand)]
        strategic_commands: WeeklyCommands,
    },
    /// Access the tactical agent
    Project
    {
        #[clap(subcommand)]
        tactical_commands: ProjectCommands,
    },
    /// Access the supervisor agents
    Daily
    {
        #[clap(subcommand)]
        supervisor_commands: DailyCommands,
    },
    /// Access the operational agents
    Operational
    {
        #[clap(subcommand)]
        operational_commands: OperationalCommands,
    },
    /// Access the SAP integration (Requires user authorization)
    Sap
    {
        #[clap(subcommand)]
        sap_commands: SapCommands,
    },
    /// Running this command will export an XLSX file for the chosen Asset
    Export
    {
        asset: Asset
    },
}

pub fn handle_command(cli: Cli, client: &Client) -> SystemMessages
{
    match cli.command {
        Commands::Status { status_commands } => match status_commands {
            StatusCommands::WorkOrders { work_orders } => match work_orders {
                WorkOrders::WorkOrderState {
                    asset,
                    level_of_detail,
                } => {
                    let orchestrator_request = OrchestratorRequest::GetWorkOrdersState(
                        asset.clone(),
                        level_of_detail.clone(),
                    );
                    SystemMessages::Orchestrator(orchestrator_request)
                }
                WorkOrders::WorkOrder {
                    work_order_number,
                    level_of_detail,
                } => {
                    let strategic_status_message = OrchestratorRequest::GetWorkOrderStatus(
                        work_order_number.into(),
                        level_of_detail.clone(),
                    );
                    SystemMessages::Orchestrator(strategic_status_message)
                }
            },
            StatusCommands::Workers => {
                todo!()
            }
            StatusCommands::Time {} => {
                todo!()
            }
            StatusCommands::Log { level } => {
                SystemMessages::Orchestrator(OrchestratorRequest::SetLogLevel(level.clone()))
            }
            StatusCommands::Profiling { level } => {
                SystemMessages::Orchestrator(OrchestratorRequest::SetProfiling(level.clone()))
            }
        },
        Commands::Orchestrator {
            orchestrator_commands,
        } => orchestrator_commands.execute(),

        Commands::Weekly { strategic_commands } => strategic_commands.execute(),

        Commands::Project { tactical_commands } => tactical_commands.execute(client),

        Commands::Daily {
            supervisor_commands,
        } => supervisor_commands.execute(client),
        Commands::Operational {
            operational_commands,
        } => operational_commands.execute(),
        Commands::Sap { sap_commands } => sap_commands.execute(),
        Commands::Export { asset } => {
            let orchestrator_request = OrchestratorRequest::Export(asset.clone());

            SystemMessages::Orchestrator(orchestrator_request)
        }
    }
}
