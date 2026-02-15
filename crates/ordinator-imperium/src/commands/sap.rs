use clap::Subcommand;
use shared_types::SystemMessages;

#[derive(Subcommand, Debug)]
pub enum SapCommands
{
    /// Extract scheduling relevant data from SAP (requires user authorization)
    ExtractFromSap,

    /// Push the 4M+ (strategic) optimized data to SAP (requires user
    /// authorization)
    PushStrategicToSap,

    /// Push the 5W (tactical) optimized data to SAP (requires user
    /// authorization)
    PushTacticalToSap,

    /// Access the 2WF (operational) optimized data (requires user authorization)
    Operational,
}

impl SapCommands
{
    pub fn execute(&self) -> SystemMessages
    {
        match self {
            SapCommands::ExtractFromSap => {
                // TODO: Implement SAP extraction with proper authorization handling
                SystemMessages::Sap
            }
            SapCommands::PushStrategicToSap => {
                todo!()
            }
            SapCommands::PushTacticalToSap => {
                todo!()
            }
            SapCommands::Operational => {
                todo!()
            }
        }
    }
}
