use clap::Subcommand;
use ordinator_scheduling_environment;

#[derive(Subcommand, Debug)]
pub enum StatusCommands
{
    WorkOrders
    {
        #[clap(subcommand)]
        work_orders: WorkOrders,
    },
    Workers,
    Time {},
}

/// Subcommands for querying work order information
#[derive(Subcommand, Debug)]
pub enum WorkOrders
{
    /// Get the aggregated state of all work orders
    WorkOrderState
    {
        asset: Asset
    },

    /// Get all details of a specific work order
    WorkOrder
    {
        work_order_number: u64
    },
}
