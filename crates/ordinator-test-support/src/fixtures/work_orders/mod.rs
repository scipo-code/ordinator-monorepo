use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

pub mod phd_work_orders_100_wo_4_resource_types;
pub mod phd_work_orders_400_wo_11_resource_types;
pub mod phd_work_orders_400_wo_4_resource_types;

pub struct WorkOrderData
{
    work_order_number: WorkOrderNumber,
    priority: WorkOrderType,
    operations: Vec<OperationInput>,
    basic_start: (i32, u32, u32),
    basic_finish: (i32, u32, u32),
    easd: (i32, u32, u32),
    lafd: (i32, u32, u32),
    codes: (bool, bool),
}

pub struct OperationInput
{
    activity: u64,
    work_remaining: f64,
    early_start: (i32, u32, u32, u32, u32, u32),
    early_finish: (i32, u32, u32, u32, u32, u32),
    preparation: f64,
    resource: Resources,
}
