use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;

pub mod phd_work_orders;
pub mod phd_work_orders_complex;
pub mod phd_work_orders_complex_11_resources;

struct WorkOrderData
{
    work_order_number: WorkOrderNumber,
    priority: WorkOrderType,

    // How does this work? You do not actually need all the diffenen
    operations: Vec<OperationInput>,
    basic_start: (u64, u64, u64),
    basic_finish: (u64, u64, u64),
    easd: (u64, u64, u64),
    lafd: (u64, u64, u64),
    codes: (bool, bool),
}

struct OperationInput
{
    activity: u64,
    work_remaining: f64,
    early_start: (u64, u64, u64, u64, u64, u64),
    early_finish: (u64, u64, u64, u64, u64, u64),
    preparation: f64,
    resource: Vec<Resources>,
}
