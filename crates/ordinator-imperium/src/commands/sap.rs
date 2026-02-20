use clap::Subcommand;
use shared_types::SystemMessages;

#[derive(Subcommand, Debug)]
pub enum SapCommands
{
    /// Extract scheduling relevant data from SAP (requires user authorization)
    ExtractFromSap,

    /// Push the 4M+ (strategic) optimized data to SAP (requires user
    /// authorization)
    PushWeeklyToSap,

    /// Push the 5W (project) optimized data to SAP (requires user
    /// authorization)
    PushProjectToSap,

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
            SapCommands::PushWeeklyToSap => {
                todo!()
            }
            SapCommands::PushProjectToSap => {
                todo!()
            }
            SapCommands::Operational => {
                todo!()
            }
        }
    }
}
